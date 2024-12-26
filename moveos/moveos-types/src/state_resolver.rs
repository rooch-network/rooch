// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::move_std::string::MoveString;
use crate::moveos_std::account::Account;
use crate::moveos_std::module_store::Package;
use crate::moveos_std::move_module::MoveModuleDynamicField;
use crate::moveos_std::object::{ObjectEntity, ObjectID, ObjectMeta, RawField};
use crate::state::{FieldKey, MoveType, ObjectState};
use crate::{
    access_path::AccessPath, h256::H256, moveos_std::object::AnnotatedObject, state::AnnotatedState,
};
use anyhow::{ensure, Error, Result};
use bytes::Bytes;
use move_binary_format::deserializer::DeserializerConfig;
use move_binary_format::errors::{
    BinaryLoaderResult, Location, PartialVMError, PartialVMResult, VMResult,
};
use move_binary_format::file_format::CompiledScript;
use move_binary_format::file_format_common::{IDENTIFIER_SIZE_MAX, VERSION_MAX};
use move_binary_format::CompiledModule;
use move_bytecode_utils::compiled_module_viewer::CompiledModuleView;
use move_core_types::identifier::{IdentStr, Identifier};
use move_core_types::metadata::Metadata;
use move_core_types::value::MoveTypeLayout;
use move_core_types::vm_status::StatusCode;
use move_core_types::{
    account_address::AccountAddress,
    language_storage::{ModuleId, StructTag, TypeTag},
};
use move_resource_viewer::{AnnotatedMoveStruct, AnnotatedMoveValue, MoveValueAnnotator};
use move_vm_runtime::{
    CodeStorage, Module, ModuleStorage, RuntimeEnvironment, Script, WithRuntimeEnvironment,
};
use move_vm_types::code::Code;
use move_vm_types::resolver::{ModuleResolver, MoveResolver, ResourceResolver};
use std::env::temp_dir;
use std::sync::Arc;

pub type StateKV = (FieldKey, ObjectState);
pub type AnnotatedStateKV = (FieldKey, AnnotatedState);

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
        key: &FieldKey,
    ) -> Result<Option<ObjectState>, anyhow::Error> {
        self.get_object(object_id).and_then(|res| {
            res.map(|obj| self.get_field_at(obj.state_root(), key))
                .unwrap_or(Ok(None))
        })
    }

    fn get_object(&self, id: &ObjectID) -> Result<Option<ObjectState>> {
        if id.is_root() {
            Ok(Some(ObjectState::new_root(self.root().clone())))
        } else {
            let field_key = id.field_key();
            let parent_id = id.parent().expect("ObjectID parent should not be None");
            let parent = self.get_object(&parent_id)?;
            match parent {
                Some(parent) => {
                    let obj_field = self.get_field_at(parent.state_root(), &field_key)?;
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
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<StateKV>, anyhow::Error> {
        let obj = self
            .get_object(object_id)?
            .ok_or_else(|| anyhow::format_err!("Object with id {} not found", object_id))?;
        self.list_fields_at(obj.state_root(), cursor, limit)
    }

    fn root(&self) -> &ObjectMeta;
}

pub struct GenesisResolver {
    root: ObjectMeta,
}

impl GenesisResolver {
    pub fn new() -> Self {
        Self {
            root: ObjectMeta::genesis_root(),
        }
    }
}

impl Default for GenesisResolver {
    fn default() -> Self {
        Self::new()
    }
}

impl StatelessResolver for GenesisResolver {
    fn get_field_at(
        &self,
        _state_root: H256,
        _key: &FieldKey,
    ) -> Result<Option<ObjectState>, anyhow::Error> {
        Ok(None)
    }

    fn list_fields_at(
        &self,
        _state_root: H256,
        _cursor: Option<FieldKey>,
        _limit: usize,
    ) -> Result<Vec<StateKV>, anyhow::Error> {
        Ok(vec![])
    }
}

impl StateResolver for GenesisResolver {
    fn root(&self) -> &ObjectMeta {
        &self.root
    }
}

impl ResourceResolver for GenesisResolver {
    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        _address: &AccountAddress,
        _resource_tag: &StructTag,
        _metadata: &[Metadata],
        _layout: Option<&MoveTypeLayout>,
    ) -> PartialVMResult<(Option<Bytes>, usize)> {
        Ok((None, 0))
    }
}

impl ModuleResolver for GenesisResolver {
    fn get_module_metadata(&self, _module_id: &ModuleId) -> Vec<Metadata> {
        vec![]
    }

    fn get_module(&self, _module_id: &ModuleId) -> PartialVMResult<Option<Bytes>> {
        Ok(None)
    }
}

pub struct RootObjectResolver<'a, R> {
    root: ObjectMeta,
    resolver: &'a R,
}

impl<'a, R> RootObjectResolver<'a, R>
where
    R: StatelessResolver,
{
    pub fn new(root: ObjectMeta, resolver: &'a R) -> Self {
        Self { root, resolver }
    }
}

impl<R> StatelessResolver for RootObjectResolver<'_, R>
where
    R: StatelessResolver,
{
    fn get_field_at(
        &self,
        state_root: H256,
        key: &FieldKey,
    ) -> Result<Option<ObjectState>, anyhow::Error> {
        self.resolver.get_field_at(state_root, key)
    }

    fn list_fields_at(
        &self,
        state_root: H256,
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<StateKV>, anyhow::Error> {
        self.resolver.list_fields_at(state_root, cursor, limit)
    }
}

impl<R> StateResolver for RootObjectResolver<'_, R>
where
    R: StatelessResolver,
{
    fn root(&self) -> &ObjectMeta {
        &self.root
    }
}

pub trait StatelessResolver {
    /// Get an object field with the key at the given state_root
    fn get_field_at(
        &self,
        state_root: H256,
        key: &FieldKey,
    ) -> Result<Option<ObjectState>, anyhow::Error>;

    /// List fields with the given state_root.
    fn list_fields_at(
        &self,
        state_root: H256,
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<StateKV>>;
}

impl<R> ResourceResolver for RootObjectResolver<'_, R>
where
    R: StatelessResolver,
{
    fn get_resource_bytes_with_metadata_and_layout(
        &self,
        address: &AccountAddress,
        resource_tag: &StructTag,
        _metadata: &[Metadata],
        _layout: Option<&MoveTypeLayout>,
    ) -> PartialVMResult<(Option<Bytes>, usize)> {
        let account_object_id = Account::account_object_id(*address);

        let key = FieldKey::derive_resource_key(resource_tag);
        let result = match self.get_field(&account_object_id, &key) {
            Ok(state_opt) => state_opt
                .map(|obj_state| {
                    ensure!(
                        obj_state.match_dynamic_field_type(
                            MoveString::type_tag(),
                            resource_tag.clone().into()
                        ),
                        "Resource type mismatch, expected field value type: {:?}, actual: {:?}",
                        resource_tag,
                        obj_state.object_type()
                    );

                    let field = RawField::parse_resource_field(
                        &obj_state.value,
                        resource_tag.clone().into(),
                    )?;
                    Ok(field.value)
                })
                .transpose(),
            Err(e) => Err(anyhow::format_err!("{:?}", e)),
        };

        match result {
            Ok(opt) => {
                if let Some(data) = opt {
                    Ok((Some(Bytes::copy_from_slice(data.as_slice())), 0))
                } else {
                    Ok((None, 0))
                }
            }
            Err(err) => Err(PartialVMError::new(StatusCode::ABORTED)),
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

    fn get_module(&self, module_id: &ModuleId) -> PartialVMResult<Option<Bytes>> {
        let package_obj_id = Package::package_id(module_id.address());
        let key = FieldKey::derive_module_key(module_id.name());
        //We wrap the modules byte codes to `MoveModule` type when store the module.
        //So we need unwrap the MoveModule type.
        match self.get_field(&package_obj_id, &key) {
            Ok(state_opt) => state_opt
                .map(|s| match s.value_as::<MoveModuleDynamicField>() {
                    Ok(v) => {
                        let ret = v.clone();
                        let x = ret.value.byte_codes.to_vec();
                        Ok(Bytes::copy_from_slice(x.as_slice()))
                    }
                    Err(e) => {
                        return Err(
                            PartialVMError::new(StatusCode::ABORTED).with_message(format!("{}", e))
                        )
                    }
                })
                .transpose(),
            Err(e) => Err(PartialVMError::new(StatusCode::ABORTED).with_message(format!("{}", e))),
        }
    }
}

impl<R> CompiledModuleView for &RootObjectResolver<'_, R>
where
    R: StatelessResolver,
{
    type Item = CompiledModule;

    fn view_compiled_module(&self, id: &ModuleId) -> Result<Option<Self::Item>> {
        Ok(match self.get_module(id)? {
            Some(bytes) => {
                let config = DeserializerConfig::new(VERSION_MAX, IDENTIFIER_SIZE_MAX);
                Some(CompiledModule::deserialize_with_config(&bytes, &config)?)
            }
            None => None,
        })
    }
}

pub trait MoveOSResolver: MoveResolver + StateResolver {}

impl<T> MoveOSResolver for T where T: MoveResolver + StateResolver {}

/// StateReader provide an unify State API with AccessPath
pub trait StateReader: StateResolver {
    /// Get states by AccessPath
    fn get_states(&self, path: AccessPath) -> Result<Vec<Option<ObjectState>>> {
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
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<StateKV>> {
        let query = path.into_state_query().into_list_query()?;
        self.list_fields(&query, cursor, limit)
    }
}

impl<R> StateReader for R where R: StateResolver {}

pub trait AnnotatedStateReader: StateReader + MoveResolver where &Self: CompiledModuleView {
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
        cursor: Option<FieldKey>,
        limit: usize,
    ) -> Result<Vec<AnnotatedStateKV>> {
        let annotator = MoveValueAnnotator::new(self);
        self.list_states(path, cursor, limit)?
            .into_iter()
            .map(|(key, state)| Ok((key, state.into_annotated_state(&annotator)?)))
            .collect::<Result<Vec<_>>>()
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

pub trait StateReaderExt: StateReader {
    fn get_account(&self, address: AccountAddress) -> Result<Option<ObjectEntity<Account>>> {
        let account_object_id = Account::account_object_id(address);
        self.get_object(&account_object_id)?
            .map(|obj| obj.into_object::<Account>())
            .transpose()
    }
}

impl<T> StateReaderExt for T where T: StateReader {}
