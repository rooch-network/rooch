// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::moveos_std::account::Account;
use crate::moveos_std::move_module::ModuleStore;
use crate::moveos_std::object::{ObjectID, RootObjectEntity};
use crate::state::{AnnotatedKeyState, KeyState};
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

pub type StateKV = (KeyState, State);
pub type AnnotatedStateKV = (AnnotatedKeyState, AnnotatedState);

/// A global state resolver which needs to be provided by the environment.
/// This allows to lookup data in remote storage.
/// If the handle is ObjectID::root(), it will get the data from the global state tree,
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
    fn resolve_object_state(&self, object_id: &ObjectID) -> Result<Option<State>, anyhow::Error> {
        let parent_id = object_id.parent().unwrap_or(ObjectID::root());
        self.resolve_table_item(&parent_id, &object_id.to_key())
    }

    fn root_object(&self) -> RootObjectEntity;
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

        let key = KeyState::from_struct_tag(tag);
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
        let key = KeyState::from_module_id(module_id);
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

    fn root_object(&self) -> RootObjectEntity {
        self.0.root_object()
    }
}

pub trait MoveOSResolver: MoveResolver + StateResolver {}

impl<T> MoveOSResolver for T where T: MoveResolver + StateResolver {}

/// StateReader provide an unify State API with AccessPath
pub trait StateReader: StateResolver {
    /// Get states by AccessPath
    fn get_states(&self, path: AccessPath) -> Result<Vec<Option<State>>> {
        let query = path.into_state_query().into_fields_query()?;
        query
            .into_iter()
            .map(|(object_id, key)| self.resolve_table_item(&object_id, &key))
            .collect()
    }

    /// List states by AccessPath
    fn list_states(
        &self,
        path: AccessPath,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        let query = path.into_state_query().into_list_query()?;
        self.list_table_items(&query, cursor, limit)
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
