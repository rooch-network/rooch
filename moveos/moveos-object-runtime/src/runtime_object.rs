// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::{
    field_value::{FieldValue, FIELD_VALUE_STRUCT_NAME},
    runtime::{
        check_type, deserialize, partial_extension_error, serialize, ERROR_OBJECT_ALREADY_BORROWED,
        ERROR_OBJECT_ALREADY_TAKEN_OUT,
    },
    TypeLayoutLoader,
};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
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
    moveos_std::{
        move_module::MoveModule,
        object::{self, ObjectEntity, ObjectID, GENESIS_STATE_ROOT},
    },
    state::{
        FieldChange, KeyState, MoveState, MoveStructState, MoveType, NormalFieldChange,
        ObjectChange, State,
    },
    state_resolver::StatelessResolver,
};
use std::collections::{btree_map::Entry, BTreeMap};

/// A structure representing runtime field.
pub enum RuntimeField {
    None(KeyState),
    Object(RuntimeObject),
    Normal(RuntimeNormalField),
}

/// A structure representing a single runtime object.
pub struct RuntimeObject {
    pub(crate) id: ObjectID,
    /// This is the Layout of ObjectEntity<T>
    pub(crate) value_layout: MoveTypeLayout,
    /// This is the TypeTag of ObjectEntity<T>
    pub(crate) value_type: TypeTag,
    /// This is the ObjectEntity<T> value in MoveVM memory
    pub(crate) value: GlobalValue,
    /// This is the ObjectEntity<T> pointer in MoveVM memory
    pub(crate) pointer: ObjectPointer,
    /// The state root of the fields
    pub(crate) state_root: H256,
    pub(crate) fields: BTreeMap<KeyState, RuntimeField>,
}

pub struct RuntimeNormalField {
    pub(crate) key: KeyState,
    /// This is the Layout of FieldValue<V>
    pub(crate) value_layout: MoveTypeLayout,
    /// This is the TypeTag of FieldValue<V>
    pub(crate) value_type: TypeTag,
    /// This is the FieldValue<V> value in MoveVM memory
    pub(crate) value: GlobalValue,
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
}

impl RuntimeNormalField {
    pub fn load(
        key: KeyState,
        value_layout: MoveTypeLayout,
        state: State,
    ) -> PartialVMResult<Self> {
        let value = deserialize(&value_layout, state.value.as_slice())?;
        //the state only contains V of FieldValue<T>, so we need wrap there.
        let wraped_value = Value::struct_(Struct::pack(vec![value]));
        let wraped_layout = MoveTypeLayout::Struct(MoveStructLayout::new(vec![value_layout]));
        let wraped_type = TypeTag::Struct(Box::new(StructTag {
            address: MOVEOS_STD_ADDRESS,
            module: object::MODULE_NAME.to_owned(),
            name: FIELD_VALUE_STRUCT_NAME.to_owned(),
            type_params: vec![state.value_type],
        }));
        Ok(Self {
            key,
            value_layout: wraped_layout,
            value_type: wraped_type,
            value: GlobalValue::cached(wraped_value)?,
        })
    }

    pub fn init(
        key: KeyState,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
        value: GlobalValue,
    ) -> PartialVMResult<Self> {
        Ok(Self {
            key,
            value_layout,
            value_type,
            value,
        })
    }

    pub fn key(&self) -> &KeyState {
        &self.key
    }

    pub fn move_to(
        &mut self,
        val: Value,
        _value_layout: MoveTypeLayout,
        value_type: TypeTag,
    ) -> PartialVMResult<()> {
        if self.value.exists()? {
            return Err(PartialVMError::new(StatusCode::RESOURCE_ALREADY_EXISTS)
                .with_message("Field already exists".to_string()));
        }
        check_type(&self.value_type, &value_type)?;
        self.value.move_to(val).map_err(|(e, _)| e)
    }

    pub fn borrow_value(&self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        check_type(&self.value_type, &expect_value_type)?;
        self.value.borrow_global()
    }

    pub fn move_from(&mut self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        check_type(&self.value_type, &expect_value_type)?;
        self.value.move_from()
    }

    pub fn as_move_module(&self) -> PartialVMResult<Option<MoveModule>> {
        if !self.value.exists()? {
            Ok(None)
        } else {
            let field_runtime_value_ref = self.borrow_value(FieldValue::<MoveModule>::type_tag())?;
            let field_runtime_value = field_runtime_value_ref
                .value_as::<Reference>()?
                .read_ref()?;
            let field_value = FieldValue::<MoveModule>::from_runtime_value(field_runtime_value)
                .map_err(|e| {
                    partial_extension_error(format!(
                        "expect FieldValue<MoveModule>, but got {:?}",
                        e
                    ))
                })?;
            Ok(Some(field_value.val))
        }
    }

    pub fn into_effect(self) -> (MoveTypeLayout, TypeTag, Option<Op<Value>>) {
        let op = self.value.into_effect();
        // we need to unwrap the FieldValue<V> to V
        let unwraped_op = op.map(|op| {
            op.map(|val| {
                let field_value_struct = val.value_as::<Struct>().expect("expect struct value");
                let mut fields = field_value_struct
                    .unpack()
                    .expect("unpack struct should success");
                fields
                    .next()
                    .expect("FieldValue<V> should have one field of type V")
            })
        });
        let unwraped_layout = match self.value_layout {
            MoveTypeLayout::Struct(layout) => layout
                .into_fields()
                .pop()
                .expect("expect FieldValue<V> must have one field"),
            _ => unreachable!("expect FieldValue<V> to be a struct"),
        };
        let unwraped_type = match self.value_type {
            TypeTag::Struct(mut tag) => tag.type_params.pop().unwrap_or_else(|| {
                panic!(
                    "expect FieldValue<V> must have one type param, value_type:{:?}",
                    tag
                )
            }),
            _ => unreachable!("expect FieldValue<V> to be a struct"),
        };
        (unwraped_layout, unwraped_type, unwraped_op)
    }

    pub fn into_change(self) -> PartialVMResult<Option<NormalFieldChange>> {
        let (layout, value_type, op) = self.into_effect();
        Ok(match op {
            Some(op) => {
                let change = op.and_then(|v| {
                    let bytes = serialize(&layout, &v)?;
                    let state = State::new(bytes, value_type);
                    Ok(state)
                })?;
                Some(NormalFieldChange { op: change })
            }
            None => None,
        })
    }
}

impl RuntimeObject {
    pub fn id(&self) -> &ObjectID {
        &self.id
    }

    pub fn load(id: ObjectID, value_layout: MoveTypeLayout, state: State) -> PartialVMResult<Self> {
        let raw_obj = state
            .as_raw_object()
            .map_err(|e| partial_extension_error(format!("expect raw object, but got {:?}", e)))?;
        let value = deserialize(&value_layout, state.value.as_slice())?;

        //If the object is system owned and not frozen or shared, it should be embeded in other struct
        //So we should make the object pointer to none, ensure no one can borrow the object pointer
        let pointer = if raw_obj.is_system_owned() && !(raw_obj.is_frozen() || raw_obj.is_shared())
        {
            ObjectPointer::none()
        } else {
            ObjectPointer::cached(id.clone())
        };
        Ok(Self {
            id: id.clone(),
            value_layout,
            value_type: state.value_type,
            value: GlobalValue::cached(value)?,
            pointer,
            state_root: raw_obj.state_root(),
            fields: Default::default(),
        })
    }

    pub fn init(
        id: ObjectID,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
        value: GlobalValue,
        state_root: H256,
    ) -> PartialVMResult<Self> {
        Ok(Self {
            id: id.clone(),
            value_layout,
            value_type,
            value,
            pointer: ObjectPointer::cached(id),
            state_root,
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
        check_type(&self.value_type, &value_type)?;
        self.value.move_to(val).map_err(|(e, _)| e)
    }

    pub fn borrow_value(&self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        check_type(&self.value_type, &expect_value_type)?;
        self.value.borrow_global()
    }

    pub fn move_from(&mut self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        check_type(&self.value_type, &expect_value_type)?;
        self.value.move_from()
    }

    pub fn borrow_pointer(&self, _expect_value_type: &TypeTag) -> PartialVMResult<Value> {
        //check_type(&self.value_type, &expect_value_type)?;
        //If the object pointer does not exist, it means the object is taken out
        if !self.pointer.value.exists()? {
            return Err(PartialVMError::new(StatusCode::ABORTED)
                .with_sub_status(ERROR_OBJECT_ALREADY_TAKEN_OUT)
                .with_message(format!("Object {} already taken out", self.id)));
        }
        //We can not distinguish between `&` and `&mut`
        //Because the GlobalValue do not distinguish between `&` and `&mut`
        //If we record a bool value to distinguish between `&` and `&mut`
        //When the `&mut` is dropped, we can not reset the bool value
        if self.pointer.value.reference_count() >= 2 {
            // We raise an error if the object is already borrowed
            // Use the error code in object.move for easy debugging
            return Err(PartialVMError::new(StatusCode::ABORTED)
                .with_sub_status(ERROR_OBJECT_ALREADY_BORROWED)
                .with_message(format!("Object {} already borrowed", self.id)));
        }
        self.pointer.value.borrow_global()
    }

    pub fn take_pointer(&mut self, _expect_value_type: &TypeTag) -> PartialVMResult<Value> {
        self.pointer.value.move_from()
    }

    pub fn return_pointer(
        &mut self,
        pointer: Value,
        _expect_value_type: &TypeTag,
    ) -> PartialVMResult<()> {
        self.pointer.value.move_to(pointer).map_err(|(e, _)| e)
    }

    /// Load a field from the object. If the field not exists, init a None field.
    pub fn load_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        key: KeyState,
    ) -> PartialVMResult<(&mut RuntimeField, Option<Option<NumBytes>>)> {
        self.load_field_with_layout_fn(resolver, key, |value_type| {
            layout_loader.get_type_layout(value_type)
        })
    }

    pub fn get_loaded_field(&self, key: &KeyState) -> Option<&RuntimeField> {
        self.fields.get(key)
    }

    pub fn load_object_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StatelessResolver,
        object_id: &ObjectID,
    ) -> PartialVMResult<(&mut RuntimeObject, Option<Option<NumBytes>>)> {
        let (field, loaded) = self.load_field(layout_loader, resolver, object_id.to_key())?;
        match field {
            RuntimeField::Object(obj) => Ok((obj, loaded)),
            RuntimeField::None(_) => Err(PartialVMError::new(StatusCode::MISSING_DATA)
                .with_message(format!("Can not load Object with id: {}", object_id))),
            _ => Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                .with_message(format!("Not Object Field with key {}", object_id))),
        }
    }

    pub fn get_loaded_object_field(
        &self,
        object_id: &ObjectID,
    ) -> PartialVMResult<Option<&RuntimeObject>> {
        let field = self.get_loaded_field(&object_id.to_key());
        match field {
            Some(RuntimeField::Object(obj)) => Ok(Some(obj)),
            Some(RuntimeField::None(_)) => Ok(None),
            None => Ok(None),
            Some(RuntimeField::Normal(field)) => {
                debug_assert!(
                    false,
                    "expect object field, but got {:?}, this should not happend",
                    field.value_type
                );
                Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                    .with_message(format!("Not Object Field with key {}", object_id)))
            }
        }
    }

    pub fn load_field_with_layout_fn(
        &mut self,
        resolver: &dyn StatelessResolver,
        key: KeyState,
        f: impl FnOnce(&TypeTag) -> PartialVMResult<MoveTypeLayout>,
    ) -> PartialVMResult<(&mut RuntimeField, Option<Option<NumBytes>>)> {
        Ok(match self.fields.entry(key.clone()) {
            Entry::Vacant(entry) => {
                let (tv, loaded) =
                    match resolver
                        .get_field_at(self.state_root, &key)
                        .map_err(|err| {
                            partial_extension_error(format!(
                                "remote object resolver failure: {}",
                                err
                            ))
                        })? {
                        Some(state) => {
                            let value_layout = f(&state.value_type)?;
                            let state_bytes_len = state.value.len() as u64;
                            (
                                RuntimeField::load(key, value_layout, state)?,
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

    pub fn into_change(self) -> PartialVMResult<Option<ObjectChange>> {
        //TODO we should process the object pointer here
        //If the object pointer is deleted, there are two case:
        //1. the object is deleted
        //2. the object pointer is taken out and not returned, tt should be embeded in other struct, we need to change the Object owener to system.

        let op = self.value.into_effect();
        let change = match op {
            Some(op) => {
                let change = op.and_then(|v| {
                    let bytes = serialize(&self.value_layout, &v)?;
                    let state = State::new(bytes, self.value_type);
                    Ok(state)
                })?;
                Some(change)
            }
            None => None,
        };

        let mut fields_change = BTreeMap::new();
        for (key, field) in self.fields.into_iter() {
            let field_change = field.into_change()?;
            if let Some(change) = field_change {
                fields_change.insert(key, change);
            }
        }
        if change.is_none() && fields_change.is_empty() {
            Ok(None)
        } else {
            Ok(Some(ObjectChange {
                op: change,
                fields: fields_change,
            }))
        }
    }

    pub fn as_object_entity<T: MoveStructState>(&self) -> PartialVMResult<ObjectEntity<T>> {
        let obj_value = self.value.borrow_global()?;
        let value_ref = obj_value.value_as::<StructRef>()?;
        ObjectEntity::<T>::from_runtime_value(value_ref.read_ref()?)
            .map_err(|_e| partial_extension_error("Convert value to ObjectEntity failed"))
    }
}

impl RuntimeField {
    pub fn field_type(&self) -> String {
        match self {
            RuntimeField::None(_) => "None".to_string(),
            RuntimeField::Object(f) => f.value_type.to_string(),
            RuntimeField::Normal(f) => f.value_type.to_string(),
        }
    }

    /// Load field from state
    pub fn load(
        key: KeyState,
        value_layout: MoveTypeLayout,
        state: State,
    ) -> PartialVMResult<Self> {
        if object::is_object_entity_type(&state.value_type) {
            let object = RuntimeObject::load(
                key.as_object_id().map_err(|e| {
                    partial_extension_error(format!("expect object id,but got {:?}, {:?}", key, e))
                })?,
                value_layout,
                state,
            )?;
            Ok(RuntimeField::Object(object))
        } else {
            let normal = RuntimeNormalField::load(key, value_layout, state)?;
            Ok(RuntimeField::Normal(normal))
        }
    }

    /// Init a field with value
    pub fn init(
        key: KeyState,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
        value: Value,
    ) -> PartialVMResult<Self> {
        // Init none GlobalValue and move value to it, make the data status is dirty
        let mut global_value = GlobalValue::none();
        global_value
            .move_to(value)
            .expect("Move value to GlobalValue none should success");

        Ok(if object::is_object_entity_type(&value_type) {
            //The object field is new, so the state_root is the genesis state root
            let state_root = *GENESIS_STATE_ROOT;
            let object = RuntimeObject::init(
                key.as_object_id().map_err(|e| {
                    partial_extension_error(format!("expect object id, but got {:?}, {:?}", key, e))
                })?,
                value_layout,
                value_type,
                global_value,
                state_root,
            )?;
            RuntimeField::Object(object)
        } else {
            let normal = RuntimeNormalField::init(key, value_layout, value_type, global_value)?;
            RuntimeField::Normal(normal)
        })
    }

    pub fn none(key: KeyState) -> Self {
        RuntimeField::None(key)
    }

    pub fn exists(&self) -> PartialVMResult<bool> {
        match self {
            RuntimeField::None(_) => Ok(false),
            RuntimeField::Object(obj) => Ok(obj.value.exists()?),
            RuntimeField::Normal(normal) => Ok(normal.value.exists()?),
        }
    }

    pub fn exists_with_type(&self, expect_value_type: TypeTag) -> PartialVMResult<bool> {
        match self {
            RuntimeField::None(_) => Ok(false),
            RuntimeField::Object(obj) => {
                Ok(obj.value.exists()? && obj.value_type == expect_value_type)
            }
            RuntimeField::Normal(normal) => {
                Ok(normal.value.exists()? && normal.value_type == expect_value_type)
            }
        }
    }

    pub fn move_to(
        &mut self,
        val: Value,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
    ) -> PartialVMResult<()> {
        match self {
            RuntimeField::None(key) => {
                *self = Self::init(key.clone(), value_layout, value_type, val)?;
                Ok(())
            }
            RuntimeField::Object(obj) => obj.move_to(val, value_layout, value_type),
            RuntimeField::Normal(normal) => normal.move_to(val, value_layout, value_type),
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
            RuntimeField::Normal(normal) => normal.borrow_value(expect_value_type),
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
            RuntimeField::Normal(normal) => normal.move_from(expect_value_type),
        }
    }

    pub fn as_move_module(&self) -> PartialVMResult<Option<MoveModule>> {
        match self {
            RuntimeField::Normal(normal) => normal.as_move_module(),
            RuntimeField::None(_) => Ok(None),
            _ => Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                .with_message("Not Module Field".to_string())),
        }
    }

    pub fn into_change(self) -> PartialVMResult<Option<FieldChange>> {
        match self {
            RuntimeField::None(_) => Ok(None),
            RuntimeField::Object(obj) => obj.into_change().map(|op| op.map(FieldChange::Object)),
            RuntimeField::Normal(normal) => {
                normal.into_change().map(|op| op.map(FieldChange::Normal))
            }
        }
    }
}
