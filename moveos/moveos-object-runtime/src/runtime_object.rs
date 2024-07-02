// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    assert_abort,
    field_value::{self, Field, FIELD_STRUCT_NAME},
    runtime::{
        check_type, deserialize, partial_extension_error, serialize, ERROR_OBJECT_ALREADY_BORROWED,
        ERROR_OBJECT_ALREADY_TAKEN_OUT_OR_EMBEDED, ERROR_OBJECT_FROZEN, ERROR_OBJECT_NOT_SHARED,
        ERROR_OBJECT_OWNER_NOT_MATCH,
    },
    TypeLayoutLoader,
};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    account_address::AccountAddress,
    effects::Op,
    gas_algebra::NumBytes,
    language_storage::{StructTag, TypeTag},
    value::{MoveStructLayout, MoveTypeLayout},
    vm_status::StatusCode,
};
use move_vm_types::values::{GlobalValue, Reference, Struct, StructRef, Value};
use moveos_types::{
    addresses::MOVEOS_STD_ADDRESS,
    h256::H256,
    move_std::string::MoveString,
    moveos_std::{
        move_module::MoveModule,
        object::{
            self, ObjectEntity, ObjectID, ObjectMeta, RawData, RawObject, GENESIS_STATE_ROOT,
        },
    },
    state::{
        FieldChange, KeyState, MoveState, MoveStructState, MoveType, NormalFieldChange,
        ObjectChange, State,
    },
    state_resolver::StatelessResolver,
};
use std::{
    collections::{btree_map::Entry, BTreeMap},
    io::Read,
};

/// A structure representing runtime field.
pub enum RuntimeField {
    /// We keep a None field to represent the field not exists
    /// Avoid repeated loading of non-existent fields
    None(AccountAddress),
    Object(RuntimeObject),
}

/// A structure representing a single runtime object.
pub struct RuntimeObject {
    //TODO should we remove the id field? because the id is in the metadata
    pub(crate) id: ObjectID,
    /// This is the Layout of T
    pub(crate) value_layout: MoveTypeLayout,
    /// This is the T value in MoveVM memory
    pub(crate) value: GlobalValue,
    /// This is the Object<T> pointer in MoveVM memory
    pub(crate) pointer: ObjectPointer,
    /// The metadata of the object
    pub(crate) metadata: ObjectMeta,
    pub(crate) fields: BTreeMap<AccountAddress, RuntimeField>,
}

/// A structure representing the `Object<T>` in Move.
/// `Object<T>` is pointer type of `ObjectEntity<T>`.
pub struct ObjectPointer {
    /// This is the Object<T> value in MoveVM memory
    pub(crate) value: GlobalValue,
}

impl ObjectPointer {
    pub fn cached(object_id: ObjectID) -> Self {
        let object_id_value = object_id.to_runtime_value();
        let value = GlobalValue::cached(Value::struct_(Struct::pack(vec![object_id_value])))
            .expect("Failed to cache the Struct");
        Self { value }
    }

    pub fn fresh(object_id: ObjectID) -> Self {
        let object_id_value = object_id.to_runtime_value();
        let mut value = GlobalValue::none();
        value
            .move_to(Value::struct_(Struct::pack(vec![object_id_value])))
            .expect("Failed to move value to GlobalValue none");
        Self { value }
    }

    pub fn none() -> Self {
        let value = GlobalValue::none();
        Self { value }
    }

    pub fn has_borrowed(&self) -> bool {
        //We can not distinguish between `&` and `&mut`
        //Because the GlobalValue do not distinguish between `&` and `&mut`
        //If we record a bool value to distinguish between `&` and `&mut`
        //When the `&mut` is dropped, we can not reset the bool value
        //The reference_count after cached is 1, so it should be 2 if borrowed
        debug_assert!(
            self.value.reference_count() <= 2,
            "The reference count should not exceed 2"
        );
        self.value.reference_count() >= 2
    }
}

impl RuntimeObject {
    pub fn id(&self) -> &ObjectID {
        &self.id
    }

    pub fn state_root(&self) -> H256 {
        self.metadata.state_root()
    }

    pub fn load(value_layout: MoveTypeLayout, raw_obj: RawObject) -> PartialVMResult<Self> {
        let (metadata, value_bytes) = raw_obj.into_metadata_and_value();
        let value = deserialize(&value_layout, &value_bytes)?;
        let id = metadata.id.clone();
        //If the object be embeded in other struct
        //So we should make the object pointer to none, ensure no one can borrow the object pointer
        let pointer = if metadata.is_embeded() {
            ObjectPointer::none()
        } else {
            ObjectPointer::cached(id.clone())
        };
        Ok(Self {
            id,
            value_layout,
            value: GlobalValue::cached(value)?,
            pointer,
            metadata,
            fields: Default::default(),
        })
    }

    pub fn init(
        id: ObjectID,
        value_layout: MoveTypeLayout,
        value: Value,
        metadata: ObjectMeta,
    ) -> PartialVMResult<Self> {
        // Init none GlobalValue and move value to it, make the data status is dirty
        let mut global_value = GlobalValue::none();
        global_value
            .move_to(value)
            .expect("Move value to GlobalValue none should success");

        Ok(Self {
            id: id.clone(),
            value_layout,
            value: global_value,
            pointer: ObjectPointer::fresh(id),
            metadata,
            fields: Default::default(),
        })
    }

    pub fn move_to(
        &mut self,
        val: Value,
        _value_layout: MoveTypeLayout,
        value_type: TypeTag,
    ) -> PartialVMResult<()> {
        if self.value.exists()? {
            return Err(PartialVMError::new(StatusCode::RESOURCE_ALREADY_EXISTS)
                .with_message("Object Field already exists".to_string()));
        }
        //check_type(&self.metadata.value_type, &value_type)?;
        self.value.move_to(val).map_err(|(e, _)| e)
    }

    pub fn borrow_value(&self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        //check_type(&self.metadata.value_type, &expect_value_type)?;
        self.value.borrow_global()
    }

    pub fn move_from(&mut self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        //check_type(&self.metadata.value_type, &expect_value_type)?;
        self.value.move_from()
    }

    pub fn borrow_pointer(&self, _expect_value_type: &TypeTag) -> PartialVMResult<Value> {
        //check_type(&self.metadata.value_type, &expect_value_type)?;

        //If the object pointer does not exist, it means the object is taken out
        assert_abort!(
            self.pointer.value.exists()?,
            ERROR_OBJECT_ALREADY_TAKEN_OUT_OR_EMBEDED,
            "Object {} already taken out",
            self.id
        );

        assert_abort!(
            !self.pointer.has_borrowed(),
            ERROR_OBJECT_ALREADY_BORROWED,
            "Object {} already borrowed",
            self.id
        );

        self.pointer.value.borrow_global()
    }

    pub fn borrow_mut_pointer(
        &self,
        owner: AccountAddress,
        by_pass_owner_check: bool,
        expect_value_type: &TypeTag,
    ) -> PartialVMResult<Value> {
        let pointer = self.borrow_pointer(expect_value_type)?;
        assert_abort!(
            !self.metadata.is_frozen(),
            ERROR_OBJECT_FROZEN,
            "Object {} is frozen",
            self.id
        );
        assert_abort!(
            self.metadata.owner == owner || by_pass_owner_check,
            ERROR_OBJECT_OWNER_NOT_MATCH,
            "Object {} is not owned by {}",
            self.id,
            owner
        );
        Ok(pointer)
    }

    pub fn borrow_mut_shared_pointer(&self, expect_value_type: &TypeTag) -> PartialVMResult<Value> {
        let pointer = self.borrow_pointer(expect_value_type)?;
        assert_abort!(
            self.metadata.is_shared(),
            ERROR_OBJECT_NOT_SHARED,
            "Object {} is not shared",
            self.id
        );
        Ok(pointer)
    }

    pub fn take_pointer(
        &mut self,
        owner: AccountAddress,
        by_pass_owner_check: bool,
        _expect_value_type: &TypeTag,
    ) -> PartialVMResult<Value> {
        //check_type(&self.metadata.value_type, &expect_value_type)?;
        assert_abort!(
            self.pointer.value.exists()?,
            ERROR_OBJECT_ALREADY_TAKEN_OUT_OR_EMBEDED,
            "Object {} already taken out",
            self.id
        );

        assert_abort!(
            !self.pointer.has_borrowed(),
            ERROR_OBJECT_ALREADY_BORROWED,
            "Object {} already borrowed",
            self.id
        );

        assert_abort!(
            self.metadata.owner == owner || by_pass_owner_check,
            ERROR_OBJECT_OWNER_NOT_MATCH,
            "Object {} is not owned by {}",
            self.id,
            owner
        );
        self.pointer.value.move_from()
    }

    pub fn take_shared_pointer(&mut self, _expect_value_type: &TypeTag) -> PartialVMResult<Value> {
        //check_type(&self.metadata.value_type, &expect_value_type)?;
        assert_abort!(
            self.pointer.value.exists()?,
            ERROR_OBJECT_ALREADY_TAKEN_OUT_OR_EMBEDED,
            "Object {} already taken out",
            self.id
        );

        assert_abort!(
            !self.pointer.has_borrowed(),
            ERROR_OBJECT_ALREADY_BORROWED,
            "Object {} already borrowed",
            self.id
        );

        assert_abort!(
            self.metadata.is_shared(),
            ERROR_OBJECT_NOT_SHARED,
            "Object {} is not shared",
            self.id
        );
        self.pointer.value.move_from()
    }

    pub fn return_pointer(
        &mut self,
        pointer: Value,
        _expect_value_type: &TypeTag,
    ) -> PartialVMResult<()> {
        debug_assert!(
            !self.pointer.value.exists()?,
            "The object pointer should not exist"
        );
        self.pointer.value.move_to(pointer).map_err(|(e, _)| e)
    }

    /// Load a field from the object. If the field not exists, init a None field.
    pub fn load_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        key: AccountAddress,
    ) -> PartialVMResult<(&mut RuntimeField, Option<Option<NumBytes>>)> {
        let state_root = self.state_root();
        Ok(match self.fields.entry(key.clone()) {
            Entry::Vacant(entry) => {
                let (tv, loaded) =
                    match resolver
                        .get_object_field_at(state_root, key)
                        .map_err(|err| {
                            partial_extension_error(format!(
                                "remote object resolver failure: {}",
                                err
                            ))
                        })? {
                        Some(raw_obj) => {
                            let value_layout =
                                layout_loader.get_type_layout(&raw_obj.value_type())?;
                            let state_bytes_len = raw_obj.value.value.len() as u64;
                            (
                                RuntimeField::load(value_layout, raw_obj)?,
                                Some(NumBytes::new(state_bytes_len)),
                            )
                        }
                        None => (RuntimeField::none(key), None),
                    };
                (entry.insert(tv), Some(loaded))
            }
            Entry::Occupied(entry) => (entry.into_mut(), None),
        })
    }

    pub fn get_loaded_field(&self, key: &AccountAddress) -> Option<&RuntimeField> {
        self.fields.get(key)
    }

    pub fn load_object_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        field_key: AccountAddress,
    ) -> PartialVMResult<(&mut RuntimeObject, Option<Option<NumBytes>>)> {
        let (field, loaded) = self.load_field(layout_loader, resolver, object_id.to_key())?;
        match field {
            RuntimeField::Object(obj) => Ok((obj, loaded)),
            RuntimeField::None(_) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!("Can not load Object with id: {}", object_id))),
        }
    }

    pub fn get_loaded_object_field(
        &self,
        field_key: AccountAddress,
    ) -> PartialVMResult<Option<&RuntimeObject>> {
        let field = self.get_loaded_field(field_key);
        match field {
            Some(RuntimeField::Object(obj)) => Ok(Some(obj)),
            Some(RuntimeField::None(_)) => Ok(None),
            None => Ok(None),
        }
    }

    pub fn into_change(self) -> PartialVMResult<Option<ObjectChange>> {
        //TODO we should process the object pointer here
        //If the object pointer is deleted, there are two case:
        //1. the object is deleted
        //2. the object pointer is taken out and not returned, tt should be embeded in other struct, we need to change the Object owener to system.

        // let op = self.value.into_effect();
        // let change = match op {
        //     Some(op) => {
        //         let change = op.and_then(|v| {
        //             let bytes = serialize(&self.value_layout, &v)?;
        //             let state = State::new(bytes, self.metadata.value_type);
        //             Ok(state)
        //         })?;
        //         Some(change)
        //     }
        //     None => None,
        // };

        // let mut fields_change = BTreeMap::new();
        // for (key, field) in self.fields.into_iter() {
        //     let field_change = field.into_change()?;
        //     if let Some(change) = field_change {
        //         fields_change.insert(key, change);
        //     }
        // }
        // if change.is_none() && fields_change.is_empty() {
        //     Ok(None)
        // } else {
        //     Ok(Some(ObjectChange {
        //         op: change,
        //         fields: fields_change,
        //     }))
        // }
        unimplemented!("into_change")
    }

    // pub fn as_object_entity<T: MoveStructState>(&self) -> PartialVMResult<ObjectEntity<T>> {
    //     let obj_value = self.value.borrow_global()?;
    //     let value_ref = obj_value.value_as::<StructRef>()?;
    //     ObjectEntity::<T>::from_runtime_value(value_ref.read_ref()?)
    //         .map_err(|_e| partial_extension_error("Convert value to ObjectEntity failed"))
    // }

    pub fn as_move_module(&self) -> PartialVMResult<Option<MoveModule>> {
        if !self.value.exists()? {
            Ok(None)
        } else {
            let field_runtime_value_ref =
                self.borrow_value(Field::<MoveString, MoveModule>::type_tag())?;
            let field_runtime_value = field_runtime_value_ref
                .value_as::<Reference>()?
                .read_ref()?;
            let field = Field::<MoveString, MoveModule>::from_runtime_value(field_runtime_value)
                .map_err(|e| {
                    partial_extension_error(format!(
                        "expect FieldValue<MoveModule>, but got {:?}",
                        e
                    ))
                })?;
            Ok(Some(field.value))
        }
    }
}

impl RuntimeField {
    pub fn field_type(&self) -> String {
        match self {
            RuntimeField::None(_) => "None".to_string(),
            RuntimeField::Object(f) => f.metadata.value_type.to_string(),
        }
    }

    /// Load field from state
    pub fn load(value_layout: MoveTypeLayout, raw_obj: RawObject) -> PartialVMResult<Self> {
        let object = RuntimeObject::load(value_layout, raw_obj)?;
        Ok(RuntimeField::Object(object))
    }

    /// Init a field with value
    pub fn init(
        parent_id: ObjectID,
        key: AccountAddress,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
        value: Value,
    ) -> PartialVMResult<Self> {
        let object_id = parent_id.child_id(key);
        //TODO update the timestamp in metadata
        let metadata = ObjectMeta::genesis_meta(object_id.clone(), value_type);
        let object = RuntimeObject::init(object_id, value_layout, value, metadata)?;
        Ok(RuntimeField::Object(object))
    }

    pub fn none(key: AccountAddress) -> Self {
        RuntimeField::None(key)
    }

    pub fn exists(&self) -> PartialVMResult<bool> {
        match self {
            RuntimeField::None(_) => Ok(false),
            RuntimeField::Object(obj) => Ok(obj.value.exists()?),
        }
    }

    pub fn exists_with_type(&self, expect_value_type: TypeTag) -> PartialVMResult<bool> {
        match self {
            RuntimeField::None(_) => Ok(false),
            RuntimeField::Object(obj) => {
                Ok(obj.value.exists()? && obj.metadata.value_type == expect_value_type)
            }
        }
    }

    pub fn move_to(
        &mut self,
        parent_id: ObjectID,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
        val: Value,
    ) -> PartialVMResult<()> {
        match self {
            RuntimeField::None(key) => {
                *self = Self::init(parent_id, *key, value_layout, value_type, val)?;
                Ok(())
            }
            RuntimeField::Object(obj) => obj.move_to(val, value_layout, value_type),
        }
    }

    pub fn borrow_value(&self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        match self {
            RuntimeField::None(_) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!(
                    "Cannot borrow value of None as type {}",
                    expect_value_type
                ))),
            RuntimeField::Object(obj) => obj.borrow_value(expect_value_type),
        }
    }

    pub fn move_from(&mut self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        match self {
            RuntimeField::None(_) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!(
                    "Cannot move value of unknown type as type {}",
                    expect_value_type
                ))),
            RuntimeField::Object(obj) => obj.move_from(expect_value_type),
        }
    }

    pub fn as_move_module(&self) -> PartialVMResult<Option<MoveModule>> {
        match self {
            RuntimeField::Object(obj) => obj.as_move_module(),
            RuntimeField::None(_) => Ok(None),
        }
    }

    pub fn into_change(self) -> PartialVMResult<Option<FieldChange>> {
        match self {
            RuntimeField::None(_) => Ok(None),
            RuntimeField::Object(obj) => obj.into_change().map(|op| op.map(FieldChange::Object)),
        }
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    use moveos_types::moveos_std::object::ObjectID;

    #[test]
    fn test_object_pointer_has_borrowed() {
        let object_id = ObjectID::random();
        let object_pointer_cached = ObjectPointer::cached(object_id.clone());
        assert_eq!(object_pointer_cached.value.reference_count(), 1);
        assert!(!object_pointer_cached.has_borrowed());
        let _borrowed_pointer = object_pointer_cached.value.borrow_global().unwrap();
        assert!(object_pointer_cached.has_borrowed());

        let object_pointer_fresh = ObjectPointer::fresh(object_id);
        assert_eq!(object_pointer_fresh.value.reference_count(), 1);
        assert!(!object_pointer_fresh.has_borrowed());
        let _borrowed_pointer = object_pointer_fresh.value.borrow_global().unwrap();
        assert!(object_pointer_fresh.has_borrowed());
    }
}
