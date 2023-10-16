// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    access_path::AccessPath,
    move_module::MoveModule,
    object::{AnnotatedObject, NamedTableID, ObjectID},
    state::{AnnotatedState, State},
};
use anyhow::{ensure, Result};
use move_core_types::{
    account_address::AccountAddress,
    identifier::IdentStr,
    language_storage::{ModuleId, StructTag, TypeTag},
    resolver::{ModuleResolver, MoveResolver, ResourceResolver},
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue, MoveValueAnnotator};

pub const GLOBAL_OBJECT_STORAGE_HANDLE: ObjectID = ObjectID::ZERO;

pub type StateKV = (Vec<u8>, State);
pub type AnnotatedStateKV = (Vec<u8>, AnnotatedState);

/// A global state resolver which needs to be provided by the environment.
/// This allows to lookup data in remote storage.
/// If the handle is GLOBAL_OBJECT_STORAGE_HANDLE, it will get the data from the global state tree,
/// otherwise it will get the data from the table state tree.
/// The key can be an ObjectID or an arbitrary key of a table.
pub trait StateResolver {
    fn resolve_table_item(
        &self,
        handle: &ObjectID,
        key: &[u8],
    ) -> Result<Option<State>, anyhow::Error>;

    fn list_table_items(
        &self,
        handle: &ObjectID,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<StateKV>, anyhow::Error>;

    // get object data from global state tree.
    fn resolve_object_state(&self, object: &ObjectID) -> Result<Option<State>, anyhow::Error> {
        self.resolve_table_item(&GLOBAL_OBJECT_STORAGE_HANDLE, &object.to_bytes())
    }
}

/// A proxy type for proxy the StateResolver to MoveResolver
/// Because the MoveResolver is a forgein trait, we can't implement the MoveResolver for StateResolver generic.
pub struct MoveOSResolverProxy<R: StateResolver>(pub R);

impl<R> ResourceResolver for MoveOSResolverProxy<R>
where
    R: StateResolver,
{
    type Error = anyhow::Error;

    fn get_resource(
        &self,
        address: &AccountAddress,
        tag: &StructTag,
    ) -> Result<Option<Vec<u8>>, Self::Error> {
        let resource_table_id = NamedTableID::Resource(*address).to_object_id();

        let key = resource_tag_to_key(tag);
        self.0
            .resolve_table_item(&resource_table_id, &key)?
            .map(|s| {
                ensure!(
                    s.match_struct_type(tag),
                    "Resource type mismatch, expected: {:?}, actual: {:?}",
                    tag,
                    s.value_type
                );
                Ok(s.value)
            })
            .transpose()
    }
}

impl<R> ModuleResolver for MoveOSResolverProxy<R>
where
    R: StateResolver,
{
    type Error = anyhow::Error;

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Self::Error> {
        let module_table_id = NamedTableID::Module(*module_id.address()).to_object_id();
        let key = module_name_to_key(module_id.name());
        //We wrap the modules byte codes to `MoveModule` type when store the module.
        //So we need unwrap the MoveModule type.
        self.0
            .resolve_table_item(&module_table_id, &key)?
            .map(|s| Ok(s.as_move_state::<MoveModule>()?.byte_codes))
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
        key: &[u8],
    ) -> Result<Option<State>, anyhow::Error> {
        self.0.resolve_table_item(handle, key)
    }

    fn list_table_items(
        &self,
        handle: &ObjectID,
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<StateKV>, anyhow::Error> {
        self.0.list_table_items(handle, cursor, limit)
    }
}

pub trait MoveOSResolver: MoveResolver<Err = anyhow::Error> + StateResolver {}

impl<T> MoveOSResolver for T where T: MoveResolver<Err = anyhow::Error> + StateResolver {}

//TODO define a ResourceKey trait to unify the resource key type, and auto impl it for ObjectID and StructTag.
pub fn resource_tag_to_key(tag: &StructTag) -> Vec<u8> {
    // The resource key is struct_tag to_canonical_string in bcs serialize format string, not String::into_bytes.
    bcs::to_bytes(&tag.to_canonical_string()).expect("bcs to_bytes String must success.")
}

pub fn module_name_to_key(name: &IdentStr) -> Vec<u8> {
    // The key is the module name in bcs serialize format string, not String::into_bytes.
    bcs::to_bytes(&name.to_string()).expect("bcs to_bytes String must success.")
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
        cursor: Option<Vec<u8>>,
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
        cursor: Option<Vec<u8>>,
        limit: usize,
    ) -> Result<Vec<AnnotatedStateKV>> {
        let annotator = MoveValueAnnotator::new(self);
        Ok(self
            .list_states(path, cursor, limit)?
            .into_iter()
            .map(|(key, state)| {
                (
                    key,
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
