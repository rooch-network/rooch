// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::move_std::ascii::MoveAsciiString;
use crate::move_std::string::MoveString;
use crate::moveos_std::account::Account;
use crate::moveos_std::move_module::ModuleStore;
use crate::moveos_std::object_id::ObjectID;
use crate::state::{AnnotatedKeyState, KeyState, MoveStructType};
use crate::{
    access_path::AccessPath,
    moveos_std::move_module::MoveModule,
    moveos_std::object::AnnotatedObject,
    state::{AnnotatedState, State},
};
use anyhow::{ensure, Error, Result};
use move_core_types::metadata::Metadata;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag, TypeTag},
    resolver::{ModuleResolver, MoveResolver, ResourceResolver},
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue, MoveValueAnnotator};

pub const GLOBAL_OBJECT_STORAGE_HANDLE: ObjectID = ObjectID::ZERO;

pub type StateKV = (KeyState, State);
pub type AnnotatedStateKV = (AnnotatedKeyState, AnnotatedState);

/// A global state resolver which needs to be provided by the environment.
/// This allows to lookup data in remote storage.
/// If the handle is GLOBAL_OBJECT_STORAGE_HANDLE, it will get the data from the global state tree,
/// otherwise it will get the data from the table state tree.
/// The key can be an ObjectID or an arbitrary key of a table.
pub trait StateResolver {
    fn resolve_table_item(
        &self,
        handle: &ObjectID,
        key: &KeyState,
    ) -> Result<Option<State>, anyhow::Error>;

    fn list_table_items(
        &self,
        handle: &ObjectID,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>, anyhow::Error>;

    // get object data from global state tree.
    fn resolve_object_state(&self, object: &ObjectID) -> Result<Option<State>, anyhow::Error> {
        self.resolve_table_item(&GLOBAL_OBJECT_STORAGE_HANDLE, &object.to_key())
    }
}

/// A proxy type for proxy the StateResolver to MoveResolver
/// Because the MoveResolver is a forgein trait, we can't implement the MoveResolver for StateResolver generic.
pub struct MoveOSResolverProxy<R: StateResolver>(pub R);

impl<R> ResourceResolver for MoveOSResolverProxy<R>
where
    R: StateResolver,
{
    fn get_resource_with_metadata(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
        _metadata: &[Metadata],
    ) -> Result<(Option<Vec<u8>>, usize), Error> {
        let account_object_id = Account::account_object_id(*address);

        let key = resource_tag_to_key(tag);
        let result = self
            .0
            .resolve_table_item(&account_object_id, &key)?
            .map(|s| {
                ensure!(
                    s.match_struct_type(tag),
                    "Resource type mismatch, expected: {:?}, actual: {:?}",
                    tag,
                    s.value_type
                );
                Ok(s.value)
            })
            .transpose();

        match result {
            Ok(opt) => {
                if let Some(data) = opt {
                    Ok((Some(data), 0))
                } else {
                    Ok((None, 0))
                }
            }
            Err(err) => Err(err),
        }
    }
}

impl<R> ModuleResolver for MoveOSResolverProxy<R>
where
    R: StateResolver,
{
    fn get_module_metadata(&self, _module_id: &ModuleId) -> Vec<Metadata> {
        vec![]
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Error> {
        let module_object_id = ModuleStore::module_store_id();
        let key = module_id_to_key(module_id);
        //We wrap the modules byte codes to `MoveModule` type when store the module.
        //So we need unwrap the MoveModule type.
        self.0
            .resolve_table_item(&module_object_id, &key)?
            .map(|s| Ok(s.cast::<MoveModule>()?.byte_codes))
            .transpose()
    }
}

impl<R> StateResolver for MoveOSResolverProxy<R>
where
    R: StateResolver,
{
    fn resolve_table_item(
        &self,
        handle: &ObjectID,
        key: &KeyState,
    ) -> Result<Option<State>, anyhow::Error> {
        self.0.resolve_table_item(handle, key)
    }

    fn list_table_items(
        &self,
        handle: &ObjectID,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>, anyhow::Error> {
        self.0.list_table_items(handle, cursor, limit)
    }
}

pub trait MoveOSResolver: MoveResolver + StateResolver {}

impl<T> MoveOSResolver for T where T: MoveResolver + StateResolver {}

//TODO define a ResourceKey trait to unify the resource key type, and auto impl it for ObjectID and StructTag.
pub fn resource_tag_to_key(tag: &StructTag) -> KeyState {
    // The resource key is struct_tag to_canonical_string in bcs serialize format string, not String::into_bytes.
    let key = bcs::to_bytes(&tag.to_canonical_string()).expect("bcs to_bytes String must success.");
    let key_type = TypeTag::Struct(Box::new(MoveAsciiString::struct_tag()));
    KeyState::new(key, key_type)
}

// pub fn module_id_to_key(module_id: &IdentStr) -> KeyState {
pub fn module_id_to_key(module_id: &ModuleId) -> KeyState {
    // The key is the module name in bcs serialize format string, not String::into_bytes.
    let key =
        bcs::to_bytes(&module_id.short_str_lossless()).expect("bcs to_bytes String must success.");
    let key_type = TypeTag::Struct(Box::new(MoveString::struct_tag()));
    KeyState::new(key, key_type)
}

/// StateReader provide an unify State API with AccessPath
pub trait StateReader: StateResolver {
    /// Get states by AccessPath
    fn get_states(&self, path: AccessPath) -> Result<Vec<Option<State>>> {
        let (handle, keys) = path.into_table_query();
        let keys = keys.ok_or_else(|| anyhow::anyhow!("AccessPath invalid path"))?;
        keys.into_iter()
            .map(|key| self.resolve_table_item(&handle, &key))
            .collect()
    }

    /// List states by AccessPath
    fn list_states(
        &self,
        path: AccessPath,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        let (handle, _keys) = path.into_table_query();
        self.list_table_items(&handle, cursor, limit)
    }
}

impl<R> StateReader for R where R: StateResolver {}

pub trait AnnotatedStateReader: StateReader + MoveResolver {
    fn get_annotated_states(&self, path: AccessPath) -> Result<Vec<Option<AnnotatedState>>> {
        let annotator = MoveValueAnnotator::new(self);
        self.get_states(path)?
            .into_iter()
            .map(|state| {
                state
                    .map(|state| state.into_annotated_state(&annotator))
                    .transpose()
            })
            .collect()
    }

    fn list_annotated_states(
        &self,
        path: AccessPath,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<AnnotatedStateKV>> {
        let annotator = MoveValueAnnotator::new(self);
        Ok(self
            .list_states(path, cursor, limit)?
            .into_iter()
            .map(|(key, state)| {
                (
                    key.into_annotated_state(&annotator)
                        .expect("key state into_annotated_state should success"),
                    state
                        .into_annotated_state(&annotator)
                        .expect("state into_annotated_state should success"),
                )
            })
            .collect::<Vec<_>>())
    }

    fn get_annotated_object(&self, object_id: ObjectID) -> Result<Option<AnnotatedObject>> {
        let annotator = MoveValueAnnotator::new(self);
        self.get_states(AccessPath::object(object_id))?
            .pop()
            .and_then(|state| state)
            .map(|state| {
                state
                    .into_annotated_state(&annotator)?
                    .into_annotated_object()
            })
            .transpose()
    }

    fn get_annotated_resource(
        &self,
        account: AccountAddress,
        resource_type: StructTag,
    ) -> Result<Option<AnnotatedState>> {
        let annotator = MoveValueAnnotator::new(self);
        self.get_states(AccessPath::resource(account, resource_type))?
            .pop()
            .and_then(|state| state)
            .map(|state| state.into_annotated_state(&annotator))
            .transpose()
    }

    fn view_value(&self, ty_tag: &TypeTag, blob: &[u8]) -> Result<AnnotatedMoveValue> {
        let annotator = MoveValueAnnotator::new(self);
        annotator.view_value(ty_tag, blob)
    }

    fn view_resource(&self, tag: &StructTag, blob: &[u8]) -> Result<AnnotatedMoveStruct> {
        let annotator = MoveValueAnnotator::new(self);
        annotator.view_resource(tag, blob)
    }
}

impl<T> AnnotatedStateReader for T where T: StateReader + MoveResolver {}
