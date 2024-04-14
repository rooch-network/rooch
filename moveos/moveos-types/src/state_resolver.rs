// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::moveos_std::account::Account;
use crate::moveos_std::move_module::ModuleStore;
use crate::moveos_std::object::{ObjectID, RawObject, RootObjectEntity};
use crate::state::{AnnotatedKeyState, KeyState};
use crate::{
    access_path::AccessPath,
    h256::H256,
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
pub trait StateResolver: StatelessResolver {
    /// Get an object field with the given object_id and key.
    fn get_field(
        &self,
        object_id: &ObjectID,
        key: &KeyState,
    ) -> Result<Option<State>, anyhow::Error> {
        self.get_object(object_id).and_then(|res| {
            res.map(|obj| self.get_field_at(obj.state_root(), key))
                .unwrap_or(Ok(None))
        })
    }

    fn get_object(&self, id: &ObjectID) -> Result<Option<RawObject>> {
        if id.is_root() {
            Ok(Some(self.root_object().to_raw()))
        } else {
            let parent_id = id.parent().expect("ObjectID parent should not be None");
            let parent = self.get_object(&parent_id)?;
            match parent {
                Some(parent) => {
                    let obj_field = self.get_object_field_at(parent.state_root(), id)?;
                    Ok(obj_field)
                }
                None => Ok(None),
            }
        }
    }

    /// List fields with the given object_id.
    fn list_fields(
        &self,
        object_id: &ObjectID,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>, anyhow::Error> {
        let obj = self
            .get_object(object_id)?
            .ok_or_else(|| anyhow::format_err!("Object with id {} not found", object_id))?;
        self.list_fields_at(obj.state_root(), cursor, limit)
    }

    fn root_object(&self) -> &RootObjectEntity;
}

pub struct RootObjectResolver<'a, R> {
    root_object: RootObjectEntity,
    resolver: &'a R,
}

impl<'a, R> RootObjectResolver<'a, R>
where
    R: StatelessResolver,
{
    pub fn new(root_object: RootObjectEntity, resolver: &'a R) -> Self {
        Self {
            root_object,
            resolver,
        }
    }
}

impl<R> StatelessResolver for RootObjectResolver<'_, R>
where
    R: StatelessResolver,
{
    fn get_field_at(
        &self,
        state_root: H256,
        key: &KeyState,
    ) -> Result<Option<State>, anyhow::Error> {
        self.resolver.get_field_at(state_root, key)
    }

    fn list_fields_at(
        &self,
        state_root: H256,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>, anyhow::Error> {
        self.resolver.list_fields_at(state_root, cursor, limit)
    }
}

impl<R> StateResolver for RootObjectResolver<'_, R>
where
    R: StatelessResolver,
{
    fn root_object(&self) -> &RootObjectEntity {
        &self.root_object
    }
}

pub trait StatelessResolver {
    /// Get an object field with the key at the given state_root
    fn get_field_at(
        &self,
        state_root: H256,
        key: &KeyState,
    ) -> Result<Option<State>, anyhow::Error>;

    fn get_object_field_at(&self, state_root: H256, id: &ObjectID) -> Result<Option<RawObject>> {
        self.get_field_at(state_root, &id.to_key())?
            .map(|state| state.as_raw_object())
            .transpose()
            .map_err(Into::into)
    }

    /// List fields with the given state_root.
    fn list_fields_at(
        &self,
        state_root: H256,
        cursor: Option<KeyState>,
        limit: usize,
    ) -> Result<Vec<StateKV>>;
}

impl<R> ResourceResolver for RootObjectResolver<'_, R>
where
    R: StatelessResolver,
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
            .get_field(&account_object_id, &key)?
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

impl<R> ModuleResolver for RootObjectResolver<'_, R>
where
    R: StatelessResolver,
{
    fn get_module_metadata(&self, _module_id: &ModuleId) -> Vec<Metadata> {
        vec![]
    }

    fn get_module(&self, module_id: &ModuleId) -> Result<Option<Vec<u8>>, Error> {
        let module_object_id = ModuleStore::module_store_id();
        let key = KeyState::from_module_id(module_id);
        //We wrap the modules byte codes to `MoveModule` type when store the module.
        //So we need unwrap the MoveModule type.
        self.get_field(&module_object_id, &key)?
            .map(|s| Ok(s.cast::<MoveModule>()?.byte_codes))
            .transpose()
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
            .map(|(object_id, key)| self.get_field(&object_id, &key))
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
        self.list_fields(&query, cursor, limit)
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
