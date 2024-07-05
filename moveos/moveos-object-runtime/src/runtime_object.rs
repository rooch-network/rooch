// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    assert_abort,
    runtime::{
        deserialize, partial_extension_error, serialize, ERROR_NOT_FOUND,
        ERROR_OBJECT_ALREADY_BORROWED, ERROR_OBJECT_ALREADY_TAKEN_OUT_OR_EMBEDED,
    },
    runtime_object_meta::RuntimeObjectMeta,
    TypeLayoutLoader,
};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    account_address::AccountAddress, effects::Op, gas_algebra::NumBytes, language_storage::TypeTag,
    value::MoveTypeLayout, vm_status::StatusCode,
};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    values::{GlobalValue, Reference, Struct, Value},
};
use moveos_types::{
    h256::H256,
    move_std::string::MoveString,
    moveos_std::{
        move_module::MoveModule,
        object::{DynamicField, ObjectID, ObjectMeta},
        timestamp::Timestamp,
    },
    state::{FieldKey, MoveState, MoveType, ObjectChange, ObjectState},
    state_resolver::StatelessResolver,
};
use std::collections::{btree_map::Entry, BTreeMap};

/// A structure representing a single runtime object.
pub struct RuntimeObject {
    pub(crate) rt_meta: RuntimeObjectMeta,
    /// This is the T value in MoveVM memory
    pub(crate) value: GlobalValue,
    /// This is the Object<T> pointer in MoveVM memory
    pub(crate) pointer: ObjectPointer,
    pub(crate) fields: BTreeMap<FieldKey, RuntimeObject>,
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

    pub fn none() -> Self {
        let value = GlobalValue::none();
        Self { value }
    }

    pub fn init(&mut self, object_id: ObjectID) -> PartialVMResult<()> {
        let object_id_value = object_id.to_runtime_value();
        self.value
            .move_to(Value::struct_(Struct::pack(vec![object_id_value])))
            .map_err(|(e, _)| e)
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
    /// Load a runtime object from the state
    pub fn load(value_layout: MoveTypeLayout, obj_state: ObjectState) -> PartialVMResult<Self> {
        let metadata = obj_state.metadata;
        let value = deserialize(&value_layout, &obj_state.value)?;
        let id = metadata.id.clone();
        //If the object be embeded in other struct
        //So we should make the object pointer to none, ensure no one can borrow the object pointer
        let pointer = if metadata.is_embeded() {
            ObjectPointer::none()
        } else {
            ObjectPointer::cached(id.clone())
        };
        Ok(Self {
            rt_meta: RuntimeObjectMeta::cached(metadata, value_layout),
            value: GlobalValue::cached(value)?,
            pointer,
            fields: Default::default(),
        })
    }

    pub fn none(obj_id: ObjectID) -> Self {
        Self {
            rt_meta: RuntimeObjectMeta::none(obj_id),
            value: GlobalValue::none(),
            pointer: ObjectPointer::none(),
            fields: Default::default(),
        }
    }

    pub fn is_none(&self) -> bool {
        self.rt_meta.is_none()
    }

    pub fn id(&self) -> &ObjectID {
        self.rt_meta.id()
    }

    pub fn state_root(&self) -> PartialVMResult<H256> {
        self.rt_meta.state_root()
    }

    pub fn metadata(&self) -> PartialVMResult<&ObjectMeta> {
        self.rt_meta.metadata()
    }

    pub fn move_to(
        &mut self,
        value: Value,
        value_type: TypeTag,
        value_layout: MoveTypeLayout,
    ) -> PartialVMResult<()> {
        if self.value.exists()? {
            return Err(PartialVMError::new(StatusCode::RESOURCE_ALREADY_EXISTS)
                .with_message("Object Field already exists".to_string()));
        }
        let obj_id = self.rt_meta.id().clone();
        self.rt_meta.init(value_type, value_layout)?;
        //If the value not exists, the pointer should also not exists
        //Because when `add_field` is called, the pointer is taken out and returned to the `native_add_field` function.
        self.pointer.init(obj_id)?;
        self.value.move_to(value).map_err(|(e, _)| e)?;
        Ok(())
    }

    pub fn borrow_value(&self, expect_value_type: Option<&TypeTag>) -> PartialVMResult<Value> {
        self.check_type(expect_value_type)?;
        self.value.borrow_global()
    }

    pub fn move_from(&mut self, expect_value_type: Option<&TypeTag>) -> PartialVMResult<Value> {
        self.check_type(expect_value_type)?;
        //Also mark the metadata as deleted or none
        self.rt_meta.move_from()?;
        //We do not need to reset the pointer, because:
        // 1. If the Object is dynamic field, the pointer is taken out and returned to the `native_add_field` function
        // 2. Otherwise, call `object::remove()` function must take out the object pointer first.
        self.value.move_from()
    }

    pub fn exists(&self) -> PartialVMResult<bool> {
        self.value.exists()
    }

    pub fn exists_with_type(&self, expect_value_type: &TypeTag) -> PartialVMResult<bool> {
        Ok(self.value.exists()? && self.rt_meta.value_type()? == expect_value_type)
    }

    /// Load a field from the object. If the field not exists, init a None field.
    pub fn load_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        field_key: FieldKey,
    ) -> PartialVMResult<(&mut RuntimeObject, Option<Option<NumBytes>>)> {
        let state_root = self.state_root()?;
        let field_obj_id = self.id().child_id(field_key);
        Ok(match self.fields.entry(field_key) {
            Entry::Vacant(entry) => {
                let (tv, loaded) =
                    match resolver
                        .get_field_at(state_root, &field_key)
                        .map_err(|err| {
                            partial_extension_error(format!(
                                "remote object resolver failure: {}",
                                err
                            ))
                        })? {
                        Some(obj_state) => {
                            debug_assert!(
                            obj_state.metadata.id == field_obj_id,
                            "The loaded object id should be equal to the expected field object id"
                        );
                            let value_layout =
                                layout_loader.get_type_layout(obj_state.value_type())?;
                            let state_bytes_len = obj_state.value.len() as u64;
                            (
                                RuntimeObject::load(value_layout, obj_state)?,
                                Some(NumBytes::new(state_bytes_len)),
                            )
                        }
                        None => (RuntimeObject::none(field_obj_id), None),
                    };
                (entry.insert(tv), Some(loaded))
            }
            Entry::Occupied(entry) => (entry.into_mut(), None),
        })
    }

    pub fn get_loaded_field(&self, field_key: &FieldKey) -> Option<&RuntimeObject> {
        self.fields
            .get(field_key)
            .filter(|rt_obj| !rt_obj.is_none())
    }

    pub fn get_mut_loaded_field(&mut self, field_key: &FieldKey) -> Option<&mut RuntimeObject> {
        self.fields
            .get_mut(field_key)
            .filter(|rt_obj| !rt_obj.is_none())
    }

    pub fn into_change(self, timestamp: &Timestamp) -> PartialVMResult<Option<ObjectChange>> {
        let object_id = self.id().clone();
        let mut rt_meta = self.rt_meta;
        let value_op = self.value.into_effect();
        let value_change = match value_op {
            Some(op) => {
                let change = op.and_then(|v| {
                    let bytes = serialize(rt_meta.value_layout()?, &v)?;
                    //If the value is changed, we update the `update_at` of the object
                    rt_meta.update_timestamp(timestamp.milliseconds)?;
                    Ok(bytes)
                })?;
                Some(change)
            }
            None => None,
        };

        let pointer_op = self.pointer.value.into_effect();
        if let Some(op) = pointer_op {
            match op {
                Op::Delete => {
                    //The object pointer is deleted, and the value is not deleted,
                    //means the object is taken out and is embeded in other struct
                    //We need to change the Object owner to system
                    if !matches!(&value_change, Some(Op::Delete)) {
                        rt_meta.to_system_owner()?;
                        if log::log_enabled!(log::Level::Trace) {
                            tracing::trace!(
                                object_id = tracing::field::display(&object_id),
                                op = "embeded",
                                "Object {} is embeded",
                                object_id
                            );
                        }
                    }
                }
                Op::New(_pointer_value) => {
                    //If the pointer is new, and the value is not new, means the enbeded object is returned
                    if !matches!(&value_change, Some(Op::New(_))) {
                        tracing::trace!(
                            object_id = tracing::field::display(&object_id),
                            op = "returned",
                            "Object {} is returned",
                            object_id
                        );
                    }
                }
                Op::Modify(_) => {
                    //if the pointer is taken out then returned, the pointer is modified
                }
            }
        };

        let mut fields_change = BTreeMap::new();
        for (key, field) in self.fields.into_iter() {
            let field_change = field.into_change(timestamp)?;
            if let Some(change) = field_change {
                fields_change.insert(key, change);
            }
        }
        let meta_change = rt_meta.into_effect();
        match meta_change {
            Some((metadata, is_dirty)) => {
                if !is_dirty && value_change.is_none() && fields_change.is_empty() {
                    Ok(None)
                } else {
                    Ok(Some(ObjectChange {
                        metadata,
                        value: value_change,
                        fields: fields_change,
                    }))
                }
            }
            None => {
                debug_assert!(
                    value_change.is_none() && fields_change.is_empty(),
                    "The object meta should not be None"
                );
                Ok(None)
            }
        }
    }

    pub fn as_move_module(&self) -> PartialVMResult<Option<MoveModule>> {
        if !self.value.exists()? {
            Ok(None)
        } else {
            let field_runtime_value_ref =
                self.borrow_value(Some(&DynamicField::<MoveString, MoveModule>::type_tag()))?;
            let field_runtime_value = field_runtime_value_ref
                .value_as::<Reference>()?
                .read_ref()?;
            let field =
                DynamicField::<MoveString, MoveModule>::from_runtime_value(field_runtime_value)
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

/// ObjectPointer functions
impl RuntimeObject {
    pub fn borrow_object(&self, expect_value_type: Option<&TypeTag>) -> PartialVMResult<Value> {
        self.check_type(expect_value_type)?;

        //If the object does not exist, it means the object is deleted
        assert_abort!(
            self.value.exists()?,
            ERROR_NOT_FOUND,
            "Object {} is not found",
            self.id()
        );

        //If the object pointer does not exist, it means the object is taken out
        assert_abort!(
            self.pointer.value.exists()?,
            ERROR_OBJECT_ALREADY_TAKEN_OUT_OR_EMBEDED,
            "Object {} already taken out",
            self.id()
        );

        assert_abort!(
            !self.pointer.has_borrowed(),
            ERROR_OBJECT_ALREADY_BORROWED,
            "Object {} already borrowed",
            self.id()
        );

        self.pointer.value.borrow_global()
    }

    pub fn take_object(&mut self, expect_value_type: Option<&TypeTag>) -> PartialVMResult<Value> {
        self.check_type(expect_value_type)?;
        assert_abort!(
            self.pointer.value.exists()?,
            ERROR_OBJECT_ALREADY_TAKEN_OUT_OR_EMBEDED,
            "Object {} already taken out",
            self.id()
        );

        assert_abort!(
            !self.pointer.has_borrowed(),
            ERROR_OBJECT_ALREADY_BORROWED,
            "Object {} already borrowed",
            self.id()
        );
        self.pointer.value.move_from()
    }

    /// Transfer the object to a new owner, the pointer is `Object<T>` in MoveVM
    pub fn transfer_object(
        &mut self,
        pointer: Value,
        new_owner: AccountAddress,
        expect_value_type: Option<&TypeTag>,
    ) -> PartialVMResult<()> {
        self.return_pointer(pointer, expect_value_type)?;
        self.rt_meta.transfer(new_owner)?;
        Ok(())
    }

    pub fn to_shared_object(
        &mut self,
        pointer: Value,
        expect_value_type: Option<&TypeTag>,
    ) -> PartialVMResult<()> {
        self.return_pointer(pointer, expect_value_type)?;
        self.rt_meta.to_shared()?;
        Ok(())
    }

    pub fn to_frozen_object(
        &mut self,
        pointer: Value,
        expect_value_type: Option<&TypeTag>,
    ) -> PartialVMResult<()> {
        self.return_pointer(pointer, expect_value_type)?;
        self.rt_meta.to_frozen()?;
        Ok(())
    }

    fn return_pointer(
        &mut self,
        pointer: Value,
        expect_value_type: Option<&TypeTag>,
    ) -> PartialVMResult<()> {
        self.check_type(expect_value_type)?;
        debug_assert!(
            !self.pointer.value.exists()?,
            "The object pointer should not exist"
        );
        self.pointer.value.move_to(pointer).map_err(|(e, _)| e)
    }
}

/// Object field functions
impl RuntimeObject {
    pub fn add_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        field_key: FieldKey,
        rt_type: &Type,
        value: Value,
    ) -> PartialVMResult<(Value, Option<Option<NumBytes>>)> {
        let value_type = layout_loader.type_to_type_tag(rt_type)?;
        let value_layout = layout_loader.get_type_layout(&value_type)?;
        let (tv, field_load_gas) = self.load_field(layout_loader, resolver, field_key)?;
        tv.move_to(value, value_type.clone(), value_layout)?;
        let object_pointer = tv.take_object(Some(&value_type))?;
        self.rt_meta.increase_size()?;
        if log::log_enabled!(log::Level::Trace) {
            tracing::trace!(
                object_id = tracing::field::display(&self.rt_meta.id()),
                op = "add_field",
                "Add field {} to Object {}",
                field_key,
                &self.rt_meta.id()
            );
        }
        Ok((object_pointer, field_load_gas))
    }

    pub fn remove_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        field_key: FieldKey,
        rt_type: &Type,
    ) -> PartialVMResult<(Value, Option<Option<NumBytes>>)> {
        let expect_value_type = layout_loader.type_to_type_tag(rt_type)?;
        let (tv, field_load_gas) = self.load_field(layout_loader, resolver, field_key)?;
        let value = tv.move_from(Some(&expect_value_type))?;
        self.rt_meta.decrease_size()?;
        if log::log_enabled!(log::Level::Trace) {
            tracing::trace!(
                object_id = tracing::field::display(self.rt_meta.id()),
                op = "remove_field",
                "Remove field {} from Object {}",
                field_key,
                self.rt_meta.id()
            );
        }
        Ok((value, field_load_gas))
    }

    pub fn borrow_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        field_key: FieldKey,
        rt_type: &Type,
    ) -> PartialVMResult<(Value, Option<Option<NumBytes>>)> {
        let expect_value_type = layout_loader.type_to_type_tag(rt_type)?;
        let (tv, field_load_gas) = self.load_field(layout_loader, resolver, field_key)?;
        let value = tv.borrow_value(Some(&expect_value_type))?;
        Ok((value, field_load_gas))
    }
}

/// Internal functions
impl RuntimeObject {
    /// Check the object type is equal to the expect type
    /// If the expect type is None, do nothing, for skip the type check
    fn check_type(&self, expect_type: Option<&TypeTag>) -> PartialVMResult<()> {
        if let Some(expect_type) = expect_type {
            let actual_type = self.rt_meta.value_type()?;
            if expect_type != actual_type {
                return Err(
                    PartialVMError::new(StatusCode::TYPE_MISMATCH).with_message(format!(
                        "RuntimeObject type {}, but get type {}",
                        actual_type, expect_type
                    )),
                );
            }
        }
        Ok(())
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

        let mut object_pointer_fresh = ObjectPointer::none();
        assert_eq!(object_pointer_fresh.value.reference_count(), 0);
        object_pointer_fresh.init(ObjectID::random()).unwrap();
        assert!(!object_pointer_fresh.has_borrowed());
        let _borrowed_pointer = object_pointer_fresh.value.borrow_global().unwrap();
        assert!(object_pointer_fresh.has_borrowed());
    }
}
