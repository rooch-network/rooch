// Copyright (c) RoochNetwork
// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use better_any::{Tid, TidAble};
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::{
    effects::Op,
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    language_storage::TypeTag,
    value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    values::{GlobalValue, Struct, StructRef, Value},
};
use moveos_object_runtime::resolved_arg::ResolvedArg;
use moveos_types::{moveos_std::object::ObjectID, state_resolver::StateResolver};
use moveos_types::{
    moveos_std::{object, tx_context::TxContext},
    state::{KeyState, MoveState},
};
use parking_lot::RwLock;
use smallvec::smallvec;
use std::{
    collections::{btree_map::Entry, BTreeMap, VecDeque},
    sync::Arc,
};

/// The native Object runtime context extension. This needs to be attached to the NativeContextExtensions
/// value which is passed into session functions, so its accessible from natives of this
/// extension.
#[derive(Tid)]
pub struct ObjectRuntimeContext<'a> {
    resolver: &'a dyn StateResolver,
    object_runtime: Arc<RwLock<ObjectRuntime>>,
}

/// Ensure the error codes in this file is consistent with the error code in object.move
const E_ALREADY_EXISTS: u64 = super::object::ERROR_ALREADY_EXISTS;
const E_NOT_FOUND: u64 = super::object::ERROR_NOT_FOUND;
const E_TYPE_MISMATCH: u64 = super::object::ERROR_TYPE_MISMATCH;

// ===========================================================================================
// Private Data Structures and Constants

pub struct TxContextValue {
    value: GlobalValue,
}

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

//TODO migrate to moveos-object-runtime crate
/// A structure representing mutable data of the ObjectRuntimeContext. This is in a RefCell
/// of the overall context so we can mutate while still accessing the overall context.
pub struct ObjectRuntime {
    pub(crate) tx_context: TxContextValue,
    objects: BTreeMap<ObjectID, RuntimeObject>,
    object_ref_in_args: BTreeMap<ObjectID, Value>,
    object_reference: BTreeMap<ObjectID, GlobalValue>,
}

/// A structure representing runtime field.
pub struct RuntimeField {
    /// This is the Layout and TypeTag of the value stored in Box<V>
    /// If the value is GlobalValue::None, the Layout and TypeTag are not known
    value_layout_and_type: Option<(MoveTypeLayout, TypeTag)>,
    /// This is the Box<V> value in MoveVM memory
    /// It can be GlobalValue::None
    box_value: GlobalValue,
}

impl RuntimeField {
    pub fn new(value_layout: MoveTypeLayout, value_type: TypeTag, box_value: GlobalValue) -> Self {
        debug_assert!(box_value.exists().unwrap());
        Self {
            value_layout_and_type: Some((value_layout, value_type)),
            box_value,
        }
    }

    pub fn none() -> Self {
        Self {
            value_layout_and_type: None,
            box_value: GlobalValue::none(),
        }
    }

    pub fn exists(&self) -> PartialVMResult<bool> {
        Ok(self.value_layout_and_type.is_some() && self.box_value.exists()?)
    }

    pub fn move_to(
        &mut self,
        val: Value,
        value_layout: MoveTypeLayout,
        value_type: TypeTag,
    ) -> Result<(), (PartialVMError, Value)> {
        //TODO extract the type check logic into a helper function
        if let Some((_exist_value_layout, exist_value_type)) = &self.value_layout_and_type {
            if *exist_value_type != value_type {
                return Err((
                    PartialVMError::new(StatusCode::TYPE_MISMATCH).with_message(format!(
                        "Cannot move value of type {} to value of type {}",
                        value_type, exist_value_type
                    )),
                    val,
                ));
            }
        } else {
            self.value_layout_and_type = Some((value_layout, value_type));
        }
        self.box_value.move_to(val)?;
        Ok(())
    }

    pub fn borrow_global(&self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        let value = self.box_value.borrow_global()?;
        match &self.value_layout_and_type {
            Some((_exist_value_layout, exist_value_type)) => {
                if *exist_value_type != expect_value_type {
                    return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH).with_message(
                        format!(
                            "Cannot borrow value of type {} as type {}",
                            exist_value_type, expect_value_type
                        ),
                    ));
                }
            }
            None => {
                //TODO ensure and test this case
                unreachable!("Cannot borrow value of unknown type")
                //return Err(PartialVMError::new(StatusCode::MISSING_DATA).with_message(format!("Cannot borrow value of unknown type as type {}", expect_value_type)));
            }
        }
        Ok(value)
    }

    pub fn move_from(&mut self, expect_value_type: TypeTag) -> PartialVMResult<Value> {
        let value = self.box_value.move_from()?;
        match &self.value_layout_and_type {
            Some((_exist_value_layout, exist_value_type)) => {
                if *exist_value_type != expect_value_type {
                    return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH).with_message(
                        format!(
                            "Cannot move value of type {} as type {}",
                            exist_value_type, expect_value_type
                        ),
                    ));
                }
            }
            None => {
                unreachable!("Cannot move value of unknown type")
            }
        }
        Ok(value)
    }

    pub fn into_effect(self) -> Option<(MoveTypeLayout, TypeTag, Op<Value>)> {
        let op_opt = self.box_value.into_effect();
        match (op_opt, self.value_layout_and_type) {
            (Some(op), Some((value_layout, value_type))) => Some((value_layout, value_type, op)),
            (None, None) => None,
            (None, Some(_)) => {
                // The box_value is loaded, but do not change, so no effect
                None
            }
            (Some(_op), None) => {
                unreachable!("Cannot have op without value_layout_and_type")
            }
        }
    }
}

/// A structure representing a single runtime object.
pub struct RuntimeObject {
    id: ObjectID,
    fields: BTreeMap<KeyState, RuntimeField>,
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

impl ObjectRuntime {
    pub fn new(tx_context: TxContext) -> Self {
        Self {
            tx_context: TxContextValue::new(tx_context),
            objects: Default::default(),
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

    /// Gets or creates a new object in the ObjectRuntime.
    pub fn get_or_create_object(
        &mut self,
        // _context: &NativeContext,
        handle: ObjectID,
    ) -> PartialVMResult<&mut RuntimeObject> {
        match self.objects.entry(handle.clone()) {
            Entry::Vacant(e) => {
                if log::log_enabled!(log::Level::Trace) {
                    log::trace!("[ObjectRuntime] creating object {}", handle);
                }
                let object = RuntimeObject {
                    id: handle,
                    fields: Default::default(),
                };
                Ok(e.insert(object))
            }
            Entry::Occupied(e) => Ok(e.into_mut()),
        }
    }

    pub fn borrow_object(&self, handle: &ObjectID) -> PartialVMResult<&RuntimeObject> {
        self.objects
            .get(handle)
            .ok_or_else(|| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }

    pub fn exist_object(&self, handle: &ObjectID) -> bool {
        self.objects.contains_key(handle)
    }

    pub fn load_object(&mut self, object_id: &ObjectID) -> VMResult<()> {
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
                self.load_object(object_id)?;
                let ref_value = self.borrow_object_reference(object_id)?;
                //We cache the object reference in the object_ref_in_args
                //Ensure the reference count and the object can not be borrowed in Move
                self.object_ref_in_args.insert(object_id.clone(), ref_value);
            }
        }
        Ok(())
    }

    /// into inner
    pub fn into_inner(self) -> (TxContext, BTreeMap<ObjectID, RuntimeObject>) {
        let ObjectRuntime {
            tx_context,
            objects,
            object_reference: _,
            object_ref_in_args: _,
        } = self;
        (tx_context.into_inner(), objects)
    }
}

impl RuntimeObject {
    fn get_or_create_global_value(
        &mut self,
        native_context: &NativeContext,
        object_context: &ObjectRuntimeContext,
        key: KeyState,
    ) -> PartialVMResult<(&mut RuntimeField, Option<Option<NumBytes>>)> {
        Ok(match self.fields.entry(key.clone()) {
            Entry::Vacant(entry) => {
                let (tv, loaded) = match object_context
                    .resolver
                    .resolve_table_item(
                        &self.id,
                        &KeyState::new(key.clone().key, key.clone().key_type),
                    )
                    .map_err(|err| {
                        partial_extension_error(format!("remote object resolver failure: {}", err))
                    })? {
                    Some(value_box) => {
                        let value_layout = get_type_layout(native_context, &value_box.value_type)?;
                        let val = deserialize_and_box(&value_layout, &value_box.value)?;
                        (
                            RuntimeField::new(
                                value_layout,
                                value_box.value_type,
                                GlobalValue::cached(val)?,
                            ),
                            Some(NumBytes::new(value_box.value.len() as u64)),
                        )
                    }
                    None => (RuntimeField::none(), None),
                };
                (entry.insert(tv), Some(loaded))
            }
            Entry::Occupied(entry) => (entry.into_mut(), None),
        })
    }

    pub fn get_or_create_global_value_with_layout_fn(
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
                        &KeyState::new(key.key.clone(), key.key_type.clone()),
                    )
                    .map_err(|err| {
                        partial_extension_error(format!("remote object resolver failure: {}", err))
                    })? {
                    Some(value_box) => {
                        let value_layout = f(&value_box.value_type)?;
                        let val = deserialize_and_box(&value_layout, &value_box.value)?;
                        (
                            RuntimeField::new(
                                value_layout,
                                value_box.value_type,
                                GlobalValue::cached(val)?,
                            ),
                            Some(NumBytes::new(value_box.value.len() as u64)),
                        )
                    }
                    None => (RuntimeField::none(), None),
                };
                (entry.insert(tv), Some(loaded))
            }
            Entry::Occupied(entry) => (entry.into_mut(), None),
        })
    }

    pub fn get_global_value(&self, key: &KeyState) -> Option<&RuntimeField> {
        self.fields.get(key)
    }

    pub fn contains_key(&self, key: &KeyState) -> bool {
        self.fields.contains_key(key)
    }

    pub fn into_inner(self) -> (ObjectID, BTreeMap<KeyState, RuntimeField>) {
        let RuntimeObject {
            id: handle,
            fields: content,
        } = self;
        (handle, content)
    }
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
}

#[derive(Debug, Clone)]
pub struct AddBoxGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_add_box(
    common_gas_params: &CommonGasParameters,
    gas_params: &AddBoxGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    //0 K Type
    //1 V Type
    //2 Box<V> Type
    assert_eq!(ty_args.len(), 3);
    assert_eq!(args.len(), 3);

    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let mut object_runtime = object_context.object_runtime.write();

    let val = args.pop_back().unwrap();
    let key = args.pop_back().unwrap();
    let handle = get_object_id(&mut args)?;

    let object = object_runtime.get_or_create_object(handle)?;
    let (tv, loaded, _, key_bytes_len) =
        get_field_runtime_value(context, object_context, object, &ty_args[0], key)?;
    let cost = gas_params.base
        + gas_params.per_byte_serialized * NumBytes::new(key_bytes_len)
        + common_gas_params.calculate_load_cost(loaded);
    let value_layout = type_to_type_layout(context, &ty_args[1])?;
    let value_type = type_to_type_tag(context, &ty_args[1])?;
    match tv.move_to(val, value_layout, value_type) {
        Ok(_) => Ok(NativeResult::ok(cost, smallvec![])),
        Err(_) => Ok(NativeResult::err(cost, E_ALREADY_EXISTS)),
    }
}

pub fn make_native_add_box(
    common_gas_params: CommonGasParameters,
    gas_params: AddBoxGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_add_box(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct BorrowBoxGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_borrow_box(
    common_gas_params: &CommonGasParameters,
    gas_params: &BorrowBoxGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(ty_args.len(), 3);
    assert_eq!(args.len(), 2);

    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let mut object_runtime = object_context.object_runtime.write();

    let key = args.pop_back().unwrap();
    let handle = get_object_id(&mut args)?;
    let object = object_runtime.get_or_create_object(handle)?;
    let (tv, loaded, field_key, key_bytes_len) =
        get_field_runtime_value(context, object_context, object, &ty_args[0], key)?;

    let cost = gas_params.base
        + gas_params.per_byte_serialized * NumBytes::new(key_bytes_len)
        + common_gas_params.calculate_load_cost(loaded);
    let value_type = type_to_type_tag(context, &ty_args[1])?;
    if tv.exists()? {
        match tv.borrow_global(value_type.clone()) {
            Ok(ref_val) => Ok(NativeResult::ok(cost, smallvec![ref_val])),
            Err(err) => {
                if log::log_enabled!(log::Level::Debug) {
                    log::warn!(
                        "[ObjectRuntime] borrow_box type mismatch: handle: {:?}, value_type: {:?} key:{:?}, err: {:?}",
                        &object.id,
                        value_type.to_canonical_string(),
                        field_key,
                    err
                    );
                }
                Ok(NativeResult::err(cost, E_TYPE_MISMATCH))
            }
        }
    } else {
        if log::log_enabled!(log::Level::Debug) {
            log::warn!(
                "[ObjectRuntime] borrow_box not found: handle: {:?}, key:{:?} not found.",
                &object.id,
                field_key
            );
        }
        Ok(NativeResult::err(cost, E_NOT_FOUND))
    }
}

pub fn make_native_borrow_box(
    common_gas_params: CommonGasParameters,
    gas_params: BorrowBoxGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_borrow_box(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct ContainsBoxGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_contains_box(
    common_gas_params: &CommonGasParameters,
    gas_params: &ContainsBoxGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(ty_args.len(), 1);
    assert_eq!(args.len(), 2);

    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let mut object_runtime = object_context.object_runtime.write();

    let key = args.pop_back().unwrap();
    let handle = get_object_id(&mut args)?;
    let object = object_runtime.get_or_create_object(handle.clone())?;
    let (tv, loaded, field_key, key_bytes_len) =
        get_field_runtime_value(context, object_context, object, &ty_args[0], key)?;

    if log::log_enabled!(log::Level::Trace) {
        log::trace!(
            "[ObjectRuntime] contains: object_id: {:?}, field key: {:?}",
            handle,
            field_key
        );
    }

    let cost = gas_params.base
        + gas_params.per_byte_serialized * NumBytes::new(key_bytes_len)
        + common_gas_params.calculate_load_cost(loaded);

    let exists = Value::bool(tv.exists()?);

    Ok(NativeResult::ok(cost, smallvec![exists]))
}

pub fn make_native_contains_box(
    common_gas_params: CommonGasParameters,
    gas_params: ContainsBoxGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_contains_box(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

fn native_contains_box_with_value_type(
    common_gas_params: &CommonGasParameters,
    gas_params: &ContainsBoxGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(ty_args.len(), 2);
    assert_eq!(args.len(), 2);

    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let mut object_runtime = object_context.object_runtime.write();

    let key = args.pop_back().unwrap();
    let handle = get_object_id(&mut args)?;
    let object = object_runtime.get_or_create_object(handle.clone())?;
    let (tv, loaded, field_key, key_bytes_len) =
        get_field_runtime_value(context, object_context, object, &ty_args[0], key)?;

    if log::log_enabled!(log::Level::Trace) {
        log::trace!(
            "[ObjectRuntime] contains: object_id: {:?}, key: {:?}",
            handle,
            field_key
        );
    }

    let cost = gas_params.base
        + gas_params.per_byte_serialized * NumBytes::new(key_bytes_len)
        + common_gas_params.calculate_load_cost(loaded);

    let value_type = type_to_type_tag(context, &ty_args[1])?;
    let exists = Value::bool(tv.borrow_global(value_type).is_ok());

    Ok(NativeResult::ok(cost, smallvec![exists]))
}

pub fn make_native_contains_box_with_value_type(
    common_gas_params: CommonGasParameters,
    gas_params: ContainsBoxGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_contains_box_with_value_type(
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
pub struct RemoveGasParameters {
    pub base: InternalGas,
    pub per_byte_serialized: InternalGasPerByte,
}

fn native_remove_box(
    common_gas_params: &CommonGasParameters,
    gas_params: &RemoveGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(ty_args.len(), 3);
    assert_eq!(args.len(), 2);

    let object_context = context.extensions().get::<ObjectRuntimeContext>();
    let mut object_runtime = object_context.object_runtime.write();

    let key = args.pop_back().unwrap();
    let handle = get_object_id(&mut args)?;
    let object = object_runtime.get_or_create_object(handle)?;

    let (tv, loaded, _, key_bytes_len) =
        get_field_runtime_value(context, object_context, object, &ty_args[0], key)?;

    let cost = gas_params.base
        + gas_params.per_byte_serialized * NumBytes::new(key_bytes_len)
        + common_gas_params.calculate_load_cost(loaded);
    let value_type = type_to_type_tag(context, &ty_args[1])?;
    if tv.exists()? {
        match tv.move_from(value_type) {
            Ok(val) => Ok(NativeResult::ok(cost, smallvec![val])),
            Err(_) => Ok(NativeResult::err(cost, E_TYPE_MISMATCH)),
        }
    } else {
        Ok(NativeResult::err(cost, E_NOT_FOUND))
    }
}

pub fn make_native_remove_box(
    common_gas_params: CommonGasParameters,
    gas_params: RemoveGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_remove_box(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
}

// =========================================================================================
// Helpers

fn get_object_id(args: &mut VecDeque<Value>) -> PartialVMResult<ObjectID> {
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

// Deserialize a value and box it to `moveos_std::raw_table::Box<V>`.
fn deserialize_and_box(layout: &MoveTypeLayout, bytes: &[u8]) -> PartialVMResult<Value> {
    let value = Value::simple_deserialize(bytes, layout).ok_or_else(|| {
        partial_extension_error(format!(
            "cannot deserialize object field or value, layout:{:?}, bytes:{:?}",
            layout,
            hex::encode(bytes)
        ))
    })?;
    Ok(Value::struct_(Struct::pack(vec![value])))
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

fn type_to_type_tag(context: &NativeContext, ty: &Type) -> PartialVMResult<TypeTag> {
    context.type_to_type_tag(ty)
}

fn get_type_layout(context: &NativeContext, type_tag: &TypeTag) -> PartialVMResult<MoveTypeLayout> {
    context
        .get_type_layout(type_tag)
        .map_err(|e| e.to_partial())
}

fn get_field_runtime_value<'a>(
    context: &NativeContext,
    object_context: &ObjectRuntimeContext,
    object: &'a mut RuntimeObject,
    key_type: &Type,
    key: Value,
) -> PartialVMResult<(
    &'a mut RuntimeField,
    Option<Option<NumBytes>>,
    KeyState,
    u64,
)> {
    let key_layout = type_to_type_layout(context, key_type)?;
    let key_type = type_to_type_tag(context, key_type)?;
    let key_bytes = serialize(&key_layout, &key)?;
    let field_key = KeyState::new(key_bytes.clone(), key_type);

    let (tv, loaded) =
        object.get_or_create_global_value(context, object_context, field_key.clone())?;

    Ok((tv, loaded, field_key, key_bytes.len() as u64))
}
