// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use better_any::{Tid, TidAble};
use mos_types::object::{Object, ObjectID, Owner};
use move_binary_format::errors::PartialVMError;
use move_core_types::{
    account_address::AccountAddress, language_storage::StructTag, value::MoveStructLayout,
    vm_status::StatusCode,
};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    values::{Struct, Value},
};
use std::{
    cell::RefCell,
    collections::{BTreeMap, BTreeSet},
};

/// A table resolver which needs to be provided by the environment. This allows to lookup
/// data in remote storage, as well as retrieve cost of table operations.
pub trait ObjectResolver {
    fn resolve_object(&self, object_id: ObjectID) -> Result<Option<Object>, anyhow::Error>;

    // fn delete_object(&self,
    //     object_id: ObjectID,
    // ) -> Result<(), anyhow::Error>;
}

pub enum TransferResult {
    New,
    SameOwner,
    OwnerChanged,
}
/// The native object context extension. This needs to be attached to the NativeContextExtensions
/// value which is passed into session functions, so its accessible from natives of this
/// extension.
#[derive(Tid)]
pub struct NativeObjectContext<'a> {
    resolver: &'a dyn ObjectResolver,
    object_data: RefCell<ObjectData>,
}

impl<'a> NativeObjectContext<'a> {
    pub fn new(resolver: &'a dyn ObjectResolver) -> Self {
        Self {
            resolver,
            object_data: RefCell::new(ObjectData::default()),
        }
    }

    pub fn get_object(&self, object_id: ObjectID) -> Result<Option<Object>, PartialVMError> {
        let object_data = self.object_data.borrow();
        //TODO need to check the transfers
        // if let Some(object) = object_data.transfers.get(&object_id) {
        //     return Ok(Some(object.clone()));
        // }
        if object_data.removed_object.contains(&object_id) {
            return Ok(None);
        }
        if object_data.new_object_ids.contains(&object_id) {
            return Ok(None);
        }
        let object = self.resolver.resolve_object(object_id).map_err(|e| {
            PartialVMError::new(StatusCode::STORAGE_ERROR).with_message(format!("{:?}", e))
        })?;
        Ok(object)
    }

    pub fn delete_id(&self, object_id: ObjectID) -> Result<(), PartialVMError> {
        let mut object_data = self.object_data.borrow_mut();
        object_data.removed_object.insert(object_id);
        Ok(())
    }

    pub(crate) fn transfer(
        &self,
        owner: Owner,
        ty: Type,
        tag: StructTag,
        layout: MoveStructLayout,
        obj: Value,
    ) -> Result<TransferResult, PartialVMError> {
        let mut object_data = self.object_data.borrow_mut();
        let object_id: ObjectID = get_object_id(obj.copy_value()?)?
            .value_as::<AccountAddress>()?
            .into();
        object_data
            .transfers
            .insert(object_id, ObjectInfo::new(owner, ty, tag, layout, obj));
        //TODO return the correct result
        Ok(TransferResult::New)
    }

    pub fn into_change_set(self) -> Result<ObjectChangeSet, PartialVMError> {
        let object_data = self.object_data.into_inner();
        //TODO process change set.
        Ok(ObjectChangeSet {
            new_objects: object_data.transfers,
        })
    }
}

#[derive(Debug)]
pub struct ObjectInfo {
    pub owner: Owner,
    pub ty: Type,
    pub tag: StructTag,
    //TODO should contains layout?
    pub layout: MoveStructLayout,
    pub value: Value,
}

impl ObjectInfo {
    pub fn new(
        owner: Owner,
        ty: Type,
        tag: StructTag,
        layout: MoveStructLayout,
        value: Value,
    ) -> Self {
        Self {
            owner,
            ty,
            tag,
            layout,
            value,
        }
    }
}

impl std::fmt::Display for ObjectInfo {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "ObjectInfo({:?}, {:?}, {:?}, {:?})",
            self.owner, self.ty, self.tag, self.value
        )
    }
}

/// A structure representing mutable data of the NativeTableContext. This is in a RefCell
/// of the overall context so we can mutate while still accessing the overall context.
#[derive(Default)]
struct ObjectData {
    new_object_ids: BTreeSet<ObjectID>,
    removed_object: BTreeSet<ObjectID>,
    transfers: BTreeMap<ObjectID, ObjectInfo>,
}

// Object { id: UID { id: ID { bytes: address } } .. }
// Extract the first field of the struct 3 times to get the id bytes.
pub fn get_object_id(object: Value) -> Result<Value, PartialVMError> {
    get_nested_struct_field(object, &[0, 0, 0])
}

// Extract a field valye that's nested inside value `v`. The offset of each nesting
// is determined by `offsets`.
pub fn get_nested_struct_field(mut v: Value, offsets: &[usize]) -> Result<Value, PartialVMError> {
    for offset in offsets {
        v = get_nth_struct_field(v, *offset)?;
    }
    Ok(v)
}

pub fn get_nth_struct_field(v: Value, n: usize) -> Result<Value, PartialVMError> {
    let mut itr = v.value_as::<Struct>()?.unpack()?;
    Ok(itr.nth(n).unwrap())
}

pub struct ObjectChangeSet {
    pub new_objects: BTreeMap<ObjectID, ObjectInfo>,
}
