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

/// A global state resolver which needs to be provided by the environment.
/// This allows to lookup data in remote storage.
/// If the handle is GLOBAL_OBJECT_STORAGE_HANDLE, it will get the data from the global state tree,
/// otherwise it will get the data from the table state tree.
/// The key can be an ObjectID or an arbitrary key of a table.
pub trait StateResolver {
    fn resolve_state(&self, handle: &ObjectID, key: &[u8]) -> Result<Option<State>, anyhow::Error>;
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
            .resolve_state(&resource_table_id, &key)?
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
            .resolve_state(&module_table_id, &key)?
            .map(|s| Ok(s.as_move_state::<MoveModule>()?.byte_codes))
            .transpose()
    }
}

impl<R> StateResolver for MoveOSResolverProxy<R>
where
    R: StateResolver,
{
    fn resolve_state(&self, handle: &ObjectID, key: &[u8]) -> Result<Option<State>, anyhow::Error> {
        self.0.resolve_state(handle, key)
    }
}

pub trait MoveOSResolver: MoveResolver + StateResolver {}

impl<T> MoveOSResolver for T where T: MoveResolver + StateResolver {}

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
        keys.into_iter()
            .map(|key| self.resolve_state(&handle, &key))
            .collect()
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
