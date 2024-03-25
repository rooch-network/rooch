// Copyright (c) RoochNetwork
// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use better_any::{Tid, TidAble};
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::{
    effects::Op,
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    ident_str,
    identifier::IdentStr,
    language_storage::{ModuleId, StructTag, TypeTag},
    value::{MoveStructLayout, MoveTypeLayout},
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    values::{GlobalValue, Reference, Struct, StructRef, Value},
};
use moveos_object_runtime::resolved_arg::ResolvedArg;
use moveos_types::{
    addresses::MOVEOS_STD_ADDRESS,
    moveos_std::{
        move_module::{ModuleStore, MoveModule},
        object::{ObjectID, RootObjectEntity},
    },
    state::{MoveStructState, MoveStructType, MoveType, State},
    state_resolver::StateResolver,
};
use moveos_types::{
    moveos_std::{object, tx_context::TxContext},
    state::{KeyState, MoveState},
};
use parking_lot::RwLock;
use serde::{Deserialize, Serialize};
use smallvec::smallvec;
use std::{
    collections::{btree_map::Entry, BTreeMap, VecDeque},
    sync::Arc,
};

/// Ensure the error codes in this file is consistent with the error code in object.move
const E_ALREADY_EXISTS: u64 = super::object::ERROR_ALREADY_EXISTS;
const E_NOT_FOUND: u64 = super::object::ERROR_NOT_FOUND;
const E_TYPE_MISMATCH: u64 = super::object::ERROR_TYPE_MISMATCH;
const E_OBJECT_RUNTIME_ERROR: u64 = super::object::ERROR_OBJECT_RUNTIME_ERROR;

const FIELD_VALUE_STRUCT_NAME: &IdentStr = ident_str!("FieldValue");

/// The native Object runtime context extension. This needs to be attached to the NativeContextExtensions
/// value which is passed into session functions, so its accessible from natives of this
/// extension.
#[derive(Tid)]
pub struct ObjectRuntimeContext<'a> {
    resolver: &'a dyn StateResolver,
    object_runtime: Arc<RwLock<ObjectRuntime>>,
}

pub struct TxContextValue {
    value: GlobalValue,
}

//TODO migrate to moveos-object-runtime crate
/// A structure representing mutable data of the ObjectRuntimeContext. This is in a RefCell
/// of the overall context so we can mutate while still accessing the overall context.
pub struct ObjectRuntime {
    pub(crate) tx_context: TxContextValue,
    //objects: BTreeMap<ObjectID, RuntimeObject>,
    root: RuntimeObject,
    object_ref_in_args: BTreeMap<ObjectID, Value>,
    object_reference: BTreeMap<ObjectID, GlobalValue>,
}

/// A wrapper of Object dynamic field value, mirroring `FieldValue<V>` in `object.move`.
#[derive(Debug, Clone, Serialize, Deserialize)]
struct FieldValue<V> {
    v: V,
}

pub struct RuntimeNormalField {
    key: KeyState,
    /// This is the Layout of FieldValue<V>
    value_layout: MoveTypeLayout,
    /// This is the TypeTag of FieldValue<V>
    value_type: TypeTag,
    /// This is the FieldValue<V> value in MoveVM memory
    value: GlobalValue,
}

/// A structure representing runtime field.
pub enum RuntimeField {
    None(KeyState),
    Object(RuntimeObject),
    Normal(RuntimeNormalField),
}

/// A structure representing a single runtime object.
pub struct RuntimeObject {
    id: ObjectID,
    /// This is the Layout of ObjectEntity<T>
    value_layout: MoveTypeLayout,
    /// This is the TypeTag of ObjectEntity<T>
    value_type: TypeTag,
    /// This is the ObjectEntity<T> value in MoveVM memory
    value: GlobalValue,
    fields: BTreeMap<KeyState, RuntimeField>,
}

pub trait TypeLayoutLoader {
    fn get_type_layout(&self, type_tag: &TypeTag) -> PartialVMResult<MoveTypeLayout>;
    fn type_to_type_layout(&self, ty: &Type) -> PartialVMResult<MoveTypeLayout>;
    fn type_to_type_tag(&self, ty: &Type) -> PartialVMResult<TypeTag>;
}

impl<'a, 'b> TypeLayoutLoader for NativeContext<'a, 'b> {
    fn get_type_layout(&self, type_tag: &TypeTag) -> PartialVMResult<MoveTypeLayout> {
        self.get_type_layout(type_tag).map_err(|e| e.to_partial())
    }
    fn type_to_type_layout(&self, ty: &Type) -> PartialVMResult<MoveTypeLayout> {
        self.type_to_type_layout(ty)?
            .ok_or_else(|| partial_extension_error("cannot determine type layout"))
    }
    fn type_to_type_tag(&self, ty: &Type) -> PartialVMResult<TypeTag> {
        self.type_to_type_tag(ty)
    }
}

// =========================================================================================
// Implementation of TxContextValue

impl TxContextValue {
    pub fn new(ctx: TxContext) -> Self {
        Self {
            value: GlobalValue::cached(ctx.to_runtime_value())
                .expect("Failed to cache the TxContext"),
        }
    }

    pub fn borrow_global(&self) -> PartialVMResult<Value> {
        self.value.borrow_global()
    }

    pub fn as_tx_context(&self) -> PartialVMResult<TxContext> {
        let value = self.value.borrow_global()?;
        let ctx_ref = value.value_as::<StructRef>()?;
        Ok(TxContext::from_runtime_value(ctx_ref.read_ref()?)
            .expect("Failed to convert Value to TxContext"))
    }

    pub fn into_inner(mut self) -> TxContext {
        let value = self
            .value
            .move_from()
            .expect("Failed to move value from GlobalValue");
        TxContext::from_runtime_value(value).expect("Failed to convert Value to TxContext")
    }
}

// =========================================================================================
// Implementation of ObjectRuntimeContext

impl<'a> ObjectRuntimeContext<'a> {
    /// Create a new instance of a object runtime context. This must be passed in via an
    /// extension into VM session functions.
    pub fn new(
        resolver: &'a dyn StateResolver,
        object_runtime: Arc<RwLock<ObjectRuntime>>,
    ) -> Self {
        Self {
            resolver,
            object_runtime,
        }
    }

    pub fn object_runtime(&self) -> Arc<RwLock<ObjectRuntime>> {
        self.object_runtime.clone()
    }
}

// =========================================================================================
// Implementation of FieldValue

impl<V> MoveStructType for FieldValue<V>
where
    V: MoveState,
{
    const ADDRESS: move_core_types::account_address::AccountAddress = MOVEOS_STD_ADDRESS;
    const MODULE_NAME: &'static IdentStr = object::MODULE_NAME;
    const STRUCT_NAME: &'static IdentStr = FIELD_VALUE_STRUCT_NAME;
}

impl<V> MoveStructState for FieldValue<V>
where
    V: MoveState,
{
    fn struct_layout() -> MoveStructLayout {
        MoveStructLayout::new(vec![V::type_layout()])
    }

    fn from_runtime_value_struct(value: Struct) -> anyhow::Result<Self>
    where
        Self: Sized,
    {
        let mut fields = value.unpack()?.collect::<Vec<Value>>();
        debug_assert!(fields.len() == 1, "Fields of FieldValue struct must be 1");
        let v = fields.pop().unwrap();
        Ok(FieldValue {
            v: V::from_runtime_value(v)?,
        })
    }
}

// =========================================================================================
// Implementation of ObjectRuntime

impl ObjectRuntime {
    pub fn new(tx_context: TxContext) -> Self {
        //TODO add root argument
        let root = RootObjectEntity::genesis_root_object();
        Self {
            tx_context: TxContextValue::new(tx_context),
            root: RuntimeObject::load(
                root.id.clone(),
                RootObjectEntity::type_layout(),
                root.into_state(),
            )
            .expect("Load root object should success"),
            object_reference: Default::default(),
            object_ref_in_args: Default::default(),
        }
    }

    pub fn tx_context(&self) -> TxContext {
        self.tx_context
            .as_tx_context()
            .expect("Failed to get tx_context")
    }

    pub fn add_to_tx_context<T: MoveState>(&mut self, value: T) -> PartialVMResult<()> {
        let mut tx_ctx = self.tx_context.as_tx_context()?;
        tx_ctx
            .add(value)
            .expect("Failed to add value to tx_context");
        self.tx_context = TxContextValue::new(tx_ctx);
        Ok(())
    }

    /// Load Object to the ObjectRuntime.
    pub fn load_object(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StateResolver,
        object_id: &ObjectID,
    ) -> PartialVMResult<(&mut RuntimeObject, Option<Option<NumBytes>>)> {
        if object_id.is_root() {
            return Ok((&mut self.root, None));
        } else {
            let parent_id = object_id.parent().expect("expect parent id");
            let (parent, parent_load_gas) =
                self.load_object(layout_loader, resolver, &parent_id)?;
            let (obj, load_gas) = parent.load_object_field(layout_loader, resolver, object_id)?;
            let total_gas = CommonGasParameters::sum_load_cost(parent_load_gas, load_gas);
            Ok((obj, total_gas))
        }
    }

    pub fn load_module(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StateResolver,
        module_id: &ModuleId,
    ) -> PartialVMResult<Option<Vec<u8>>> {
        let module_store_id = ModuleStore::module_store_id();
        let (module_store_obj, _) = self.load_object(layout_loader, resolver, &module_store_id)?;
        let key = KeyState::from_module_id(module_id);
        let (module_field, _loaded) = module_store_obj.load_field(layout_loader, resolver, key)?;
        let move_module = module_field.as_move_module()?;
        Ok(move_module.map(|m| m.byte_codes))
    }

    pub fn publish_module(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StateResolver,
        module_id: &ModuleId,
        blob: Vec<u8>,
        is_republishing: bool,
    ) -> PartialVMResult<()> {
        let module_store_id = ModuleStore::module_store_id();
        let (module_store_obj, _) = self.load_object(layout_loader, resolver, &module_store_id)?;
        let key = KeyState::from_module_id(module_id);
        let (module_field, _) = module_store_obj.load_field(layout_loader, resolver, key)?;

        let value_type = FieldValue::<MoveModule>::type_tag();
        let value_layout = FieldValue::<MoveModule>::type_layout();

        let move_module = MoveModule::new(blob);
        let field_value = FieldValue { v: move_module };
        let runtime_field_value = field_value.to_runtime_value();
        if is_republishing {
            let _old_value = module_field.move_from(value_type.clone())?;
        }

        module_field.move_to(runtime_field_value, value_layout, value_type)?;

        Ok(())
    }

    pub fn exists_module(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StateResolver,
        module_id: &ModuleId,
    ) -> PartialVMResult<bool> {
        let module_store_id = ModuleStore::module_store_id();
        let (module_store_obj, _) = self.load_object(layout_loader, resolver, &module_store_id)?;
        let key = KeyState::from_module_id(module_id);
        let (field, _) = module_store_obj.load_field(layout_loader, resolver, key)?;
        Ok(field.exists_with_type(FieldValue::<MoveModule>::type_tag())?)
    }

    pub fn load_object_reference(&mut self, object_id: &ObjectID) -> VMResult<()> {
        self.object_reference
            .entry(object_id.clone())
            .or_insert_with(|| {
                //TODO we should load the ObjectEntity<T> from the resolver
                //Then cache the Object<T>
                let object_id_value = object_id.to_runtime_value();
                GlobalValue::cached(Value::struct_(Struct::pack(vec![object_id_value])))
                    .expect("Failed to cache the Struct")
            });
        Ok(())
    }

    /// Borrow &Object<T> or &mut Object<T>
    pub fn borrow_object_reference(&mut self, object_id: &ObjectID) -> VMResult<Value> {
        let gv = self.object_reference.get(object_id).ok_or_else(|| {
            PartialVMError::new(StatusCode::STORAGE_ERROR).finish(Location::Undefined)
        })?;

        if gv.reference_count() >= 2 {
            // We raise an error if the object is already borrowed
            // Use the error code in object.move for easy debugging
            return Err(PartialVMError::new(StatusCode::ABORTED)
                .with_sub_status(super::object::ERROR_OBJECT_ALREADY_BORROWED)
                .with_message(format!("Object {} already borrowed", object_id))
                .finish(Location::Module(object::MODULE_ID.clone())));
        }

        gv.borrow_global()
            .map_err(|e| e.finish(Location::Undefined))
    }

    pub fn load_arguments(&mut self, resovled_args: &[ResolvedArg]) -> VMResult<()> {
        for resolved_arg in resovled_args {
            if let ResolvedArg::Object(object_arg) = resolved_arg {
                let object_id = object_arg.object_id();
                self.load_object_reference(object_id)?;
                let ref_value = self.borrow_object_reference(object_id)?;
                //We cache the object reference in the object_ref_in_args
                //Ensure the reference count and the object can not be borrowed in Move
                self.object_ref_in_args.insert(object_id.clone(), ref_value);
            }
        }
        Ok(())
    }

    // into inner
    pub fn into_inner(self) -> (TxContext, RuntimeObject) {
        let ObjectRuntime {
            tx_context,
            root,
            object_reference: _,
            object_ref_in_args: _,
        } = self;
        (tx_context.into_inner(), root)
    }
}

// =========================================================================================
// Implementation of RuntimeField

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
        value: Value,
    ) -> PartialVMResult<Self> {
        Ok(Self {
            key,
            value_layout,
            value_type,
            value: GlobalValue::cached(value)?,
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
            Ok(Some(field_value.v))
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
            TypeTag::Struct(mut tag) => tag
                .type_params
                .pop()
                .expect("expect FieldValue<V> must have one type param"),
            _ => unreachable!("expect FieldValue<V> to be a struct"),
        };
        (unwraped_layout, unwraped_type, unwraped_op)
    }
}

impl RuntimeField {
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
        Ok(if object::is_object_entity_type(&value_type) {
            let object = RuntimeObject::init(
                key.as_object_id().map_err(|e| {
                    partial_extension_error(format!("expect object id, but got {:?}, {:?}", key, e))
                })?,
                value_layout,
                value_type,
                value,
            )?;
            RuntimeField::Object(object)
        } else {
            let normal = RuntimeNormalField::init(key, value_layout, value_type, value)?;
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
            RuntimeField::None(_) => {
                return Err(
                    PartialVMError::new(StatusCode::MISSING_DATA).with_message(format!(
                        "Cannot borrow value of None as type {}",
                        expect_value_type
                    )),
                );
            }
            RuntimeField::Object(obj) => obj.borrow_value(expect_value_type),
            RuntimeField::Normal(normal) => normal.borrow_value(expect_value_type),
        }
    }

    pub fn move_from(&mut self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        match self {
            RuntimeField::None(_) => {
                return Err(
                    PartialVMError::new(StatusCode::MISSING_DATA).with_message(format!(
                        "Cannot move value of unknown type as type {}",
                        expect_value_type
                    )),
                );
            }
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

    pub fn into_effect(self) -> Option<(MoveTypeLayout, TypeTag, Op<Value>)> {
        match self {
            RuntimeField::None(_) => None,
            RuntimeField::Object(obj) => {
                let (layout, value_type, op) = obj.into_effect();
                op.map(|op| (layout, value_type, op))
            }
            RuntimeField::Normal(normal) => {
                let (layout, value_type, op) = normal.into_effect();
                op.map(|op| (layout, value_type, op))
            }
        }
    }
}

// =========================================================================================
// Implementation of RuntimeObject

impl RuntimeObject {
    pub fn load(id: ObjectID, value_layout: MoveTypeLayout, state: State) -> PartialVMResult<Self> {
        let value = deserialize(&value_layout, state.value.as_slice())?;
        Ok(Self {
            id,
            value_layout,
            value_type: state.value_type,
            value: GlobalValue::cached(value)?,
            fields: Default::default(),
        })
    }

    pub fn init(
        id: ObjectID,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
        value: Value,
    ) -> PartialVMResult<Self> {
        Ok(Self {
            id,
            value_layout,
            value_type,
            value: GlobalValue::cached(value)?,
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

    /// Load a field from the object. If the field not exists, init a None field.
    pub fn load_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StateResolver,
        key: KeyState,
    ) -> PartialVMResult<(&mut RuntimeField, Option<Option<NumBytes>>)> {
        self.load_field_with_layout_fn(resolver, key, |value_type| {
            layout_loader.get_type_layout(value_type)
        })
    }

    pub fn load_object_field(
        &mut self,
        layout_loader: &dyn TypeLayoutLoader,
        resolver: &dyn StateResolver,
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

    pub fn load_field_with_layout_fn(
        &mut self,
        resolver: &dyn StateResolver,
        key: KeyState,
        f: impl FnOnce(&TypeTag) -> PartialVMResult<MoveTypeLayout>,
    ) -> PartialVMResult<(&mut RuntimeField, Option<Option<NumBytes>>)> {
        Ok(match self.fields.entry(key.clone()) {
            Entry::Vacant(entry) => {
                let (tv, loaded) = match resolver
                    .resolve_table_item(
                        &self.id,
                        &KeyState::new(key.clone().key, key.clone().key_type),
                    )
                    .map_err(|err| {
                        partial_extension_error(format!("remote object resolver failure: {}", err))
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

    /// Get a loaded field from the object, do not check the storage.
    pub fn get_loaded_field(&self, key: &KeyState) -> Option<&RuntimeField> {
        self.fields.get(key)
    }

    /// Check if a field is loaded in the object.
    pub fn contains_loaded_field(&self, key: &KeyState) -> bool {
        self.fields.contains_key(key)
    }

    pub fn into_effect(self) -> (MoveTypeLayout, TypeTag, Option<Op<Value>>) {
        let op = self.value.into_effect();
        (self.value_layout, self.value_type, op)
    }

    // pub fn into_inner(self) -> (ObjectID, BTreeMap<KeyState, RuntimeField>) {
    //     let RuntimeObject {
    //         id: handle,
    //         value,
    //         fields: content,
    //     } = self;
    //     (handle, content)
    // }
}

// =========================================================================================
// Native Function Implementations

#[derive(Debug, Clone)]
pub struct CommonGasParameters {
    pub load_base: InternalGas,
    pub load_per_byte: InternalGasPerByte,
    pub load_failure: InternalGas,
}

impl CommonGasParameters {
    fn calculate_load_cost(&self, loaded: Option<Option<NumBytes>>) -> InternalGas {
        self.load_base
            + match loaded {
                Some(Some(num_bytes)) => self.load_per_byte * num_bytes,
                Some(None) => self.load_failure,
                None => 0.into(),
            }
    }

    fn sum_load_cost(
        first_loaded: Option<Option<NumBytes>>,
        second_loaded: Option<Option<NumBytes>>,
    ) -> Option<Option<NumBytes>> {
        match (first_loaded, second_loaded) {
            (Some(Some(first)), Some(Some(second))) => Some(Some(first + second)),
            (Some(Some(first)), None) => Some(Some(first)),
            (None, Some(Some(second))) => Some(Some(second)),
            (Some(None), _) | (_, Some(None)) => Some(None),
            _ => None,
        }
    }
}

fn native_fn_dispatch(
    common_gas_params: &CommonGasParameters,
    base: InternalGas,
    per_byte_serialized: InternalGasPerByte,
    context: &mut NativeContext,
    object_id: ObjectID,
    field_key: KeyState,
    f: impl FnOnce(&dyn TypeLayoutLoader, &mut RuntimeField) -> PartialVMResult<Option<Value>>,
) -> PartialVMResult<NativeResult> {
    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let mut object_runtime = object_context.object_runtime.write();

    let (object, object_load_gas) =
        object_runtime.load_object(context, object_context.resolver, &object_id)?;
    let field_key_bytes = field_key.key.len() as u64;
    let (tv, field_load_gas) =
        object.load_field(context, object_context.resolver, field_key.clone())?;
    let gas_cost = base
        + per_byte_serialized * NumBytes::new(field_key_bytes)
        + common_gas_params.calculate_load_cost(object_load_gas)
        + common_gas_params.calculate_load_cost(field_load_gas);

    let result = f(context, tv);
    match result {
        Ok(ret) => Ok(NativeResult::ok(
            gas_cost,
            ret.map(|v| smallvec![v]).unwrap_or(smallvec![]),
        )),
        Err(err) => {
            let abort_code = match err.major_status() {
                StatusCode::MISSING_DATA => E_NOT_FOUND,
                StatusCode::TYPE_MISMATCH => E_TYPE_MISMATCH,
                StatusCode::RESOURCE_ALREADY_EXISTS => E_ALREADY_EXISTS,
                _ => E_OBJECT_RUNTIME_ERROR,
            };
            if log::log_enabled!(log::Level::Debug) {
                log::warn!(
                    "[ObjectRuntime] native_function error: object_id: {:?}, key:{}, err: {:?}, abort: {}",
                    object_id,
                    field_key,
                    err,
                    abort_code
                );
            };
            Ok(NativeResult::err(gas_cost, abort_code))
        }
    }
}

fn native_borrow_root(
    common_gas_params: &CommonGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 0);
    debug_assert_eq!(args.len(), 0);
    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let object_runtime = object_context.object_runtime.write();
    let value = object_runtime
        .root
        .borrow_value(RootObjectEntity::type_tag())?;
    let gas_cost = common_gas_params.load_base;
    Ok(NativeResult::ok(gas_cost, smallvec![value]))
}

pub fn make_native_borrow_root(common_gas_params: CommonGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_borrow_root(&common_gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct AddFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_add_field(
    common_gas_params: &CommonGasParameters,
    gas_params: &AddFieldGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    //0 K Type
    //1 V Type FieldValue or ObjectEntity

    debug_assert_eq!(ty_args.len(), 2);
    debug_assert_eq!(args.len(), 3);

    let val = args.pop_back().unwrap();
    let key = args.pop_back().unwrap();
    let object_id = pop_object_id(&mut args)?;

    let field_key = serialize_key(context, &ty_args[0], key)?;

    native_fn_dispatch(
        common_gas_params,
        gas_params.base,
        gas_params.per_byte_serialized,
        context,
        object_id,
        field_key,
        move |layout_loader, field| {
            let value_layout = layout_loader.type_to_type_layout(&ty_args[1])?;
            let value_type = layout_loader.type_to_type_tag(&ty_args[1])?;
            field.move_to(val, value_layout, value_type).map(|_| None)
        },
    )
}

pub fn make_native_add_field(
    common_gas_params: CommonGasParameters,
    gas_params: AddFieldGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_add_field(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct BorrowFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_borrow_field(
    common_gas_params: &CommonGasParameters,
    gas_params: &BorrowFieldGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 2);
    debug_assert_eq!(args.len(), 2);

    let key = args.pop_back().unwrap();
    let object_id = pop_object_id(&mut args)?;

    let field_key = serialize_key(context, &ty_args[0], key)?;

    native_fn_dispatch(
        common_gas_params,
        gas_params.base,
        gas_params.per_byte_serialized,
        context,
        object_id,
        field_key,
        |layout_loader, field| {
            let value_type = layout_loader.type_to_type_tag(&ty_args[1])?;
            field.borrow_value(value_type).map(|v| Some(v))
        },
    )
}

pub fn make_native_borrow_field(
    common_gas_params: CommonGasParameters,
    gas_params: BorrowFieldGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_borrow_field(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct ContainsFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_contains_field(
    common_gas_params: &CommonGasParameters,
    gas_params: &ContainsFieldGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 2);

    let key = args.pop_back().unwrap();
    let object_id = pop_object_id(&mut args)?;

    let field_key = serialize_key(context, &ty_args[0], key)?;

    native_fn_dispatch(
        common_gas_params,
        gas_params.base,
        gas_params.per_byte_serialized,
        context,
        object_id,
        field_key,
        |_layout_loader, field| Ok(Some(Value::bool(field.exists()?))),
    )
}

pub fn make_native_contains_field(
    common_gas_params: CommonGasParameters,
    gas_params: ContainsFieldGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_contains_field(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

fn native_contains_field_with_value_type(
    common_gas_params: &CommonGasParameters,
    gas_params: &ContainsFieldGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 2);
    debug_assert_eq!(args.len(), 2);

    let key = args.pop_back().unwrap();
    let object_id = pop_object_id(&mut args)?;

    let field_key = serialize_key(context, &ty_args[0], key)?;

    native_fn_dispatch(
        common_gas_params,
        gas_params.base,
        gas_params.per_byte_serialized,
        context,
        object_id,
        field_key,
        |layout_loader, field| {
            let value_type = layout_loader.type_to_type_tag(&ty_args[1])?;
            Ok(Some(Value::bool(field.exists_with_type(value_type)?)))
        },
    )
}

pub fn make_native_contains_field_with_value_type(
    common_gas_params: CommonGasParameters,
    gas_params: ContainsFieldGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_contains_field_with_value_type(
                &common_gas_params,
                &gas_params,
                context,
                ty_args,
                args,
            )
        },
    )
}

#[derive(Debug, Clone)]
pub struct RemoveFieldGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_remove_field(
    common_gas_params: &CommonGasParameters,
    gas_params: &RemoveFieldGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 2);
    debug_assert_eq!(args.len(), 2);

    let key = args.pop_back().unwrap();
    let object_id = pop_object_id(&mut args)?;

    let field_key = serialize_key(context, &ty_args[0], key)?;

    native_fn_dispatch(
        common_gas_params,
        gas_params.base,
        gas_params.per_byte_serialized,
        context,
        object_id,
        field_key,
        |layout_loader, field| {
            let value_type = layout_loader.type_to_type_tag(&ty_args[1])?;
            field.move_from(value_type).map(|v| Some(v))
        },
    )
}

pub fn make_native_remove_field(
    common_gas_params: CommonGasParameters,
    gas_params: RemoveFieldGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_remove_field(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

// =========================================================================================
// Helpers

fn pop_object_id(args: &mut VecDeque<Value>) -> PartialVMResult<ObjectID> {
    let handle = args.pop_back().unwrap();
    ObjectID::from_runtime_value(handle).map_err(|e| {
        if log::log_enabled!(log::Level::Debug) {
            log::warn!("[ObjectRuntime] get_object_id: {:?}", e);
        }
        PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(e.to_string())
    })
}

pub fn serialize(layout: &MoveTypeLayout, val: &Value) -> PartialVMResult<Vec<u8>> {
    val.simple_serialize(layout).ok_or_else(|| {
        partial_extension_error(format!(
            "cannot serialize object field or value, layout:{:?}, val:{:?}",
            layout, val
        ))
    })
}

fn deserialize(layout: &MoveTypeLayout, bytes: &[u8]) -> PartialVMResult<Value> {
    let value = Value::simple_deserialize(bytes, layout).ok_or_else(|| {
        partial_extension_error(format!(
            "cannot deserialize object field or value, layout:{:?}, bytes:{:?}",
            layout,
            hex::encode(bytes)
        ))
    })?;
    Ok(value)
}

fn partial_extension_error(msg: impl ToString) -> PartialVMError {
    log::debug!("PartialVMError: {}", msg.to_string());
    PartialVMError::new(StatusCode::VM_EXTENSION_ERROR).with_message(msg.to_string())
}

fn type_to_type_layout(context: &NativeContext, ty: &Type) -> PartialVMResult<MoveTypeLayout> {
    context
        .type_to_type_layout(ty)?
        .ok_or_else(|| partial_extension_error("cannot determine type layout"))
}

fn serialize_key(
    context: &NativeContext,
    key_type: &Type,
    key: Value,
) -> PartialVMResult<KeyState> {
    let key_layout = type_to_type_layout(context, key_type)?;
    let key_type_tag = context.type_to_type_tag(key_type)?;
    let key_bytes = serialize(&key_layout, &key)?;
    Ok(KeyState::new(key_bytes, key_type_tag))
}

fn check_type(actual_type: &TypeTag, expect_type: &TypeTag) -> PartialVMResult<()> {
    if expect_type != actual_type {
        return Err(
            PartialVMError::new(StatusCode::TYPE_MISMATCH).with_message(format!(
                "Field state type {}, but get type {}",
                actual_type, expect_type
            )),
        );
    }
    Ok(())
}
