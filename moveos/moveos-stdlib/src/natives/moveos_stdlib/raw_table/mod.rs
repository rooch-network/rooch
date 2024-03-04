// Copyright (c) RoochNetwork
// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

/// A native Table implementation for save any type of value.
/// Refactor from https://github.com/rooch-network/move/blob/c7d8c2b0cdd06dbd90e0ab306932356620b5648a/language/extensions/move-table-extension/src/lib.rs#L4
use better_any::{Tid, TidAble};
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::{
    account_address::AccountAddress,
    effects::Op,
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    language_storage::TypeTag,
    value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_runtime::{
    native_functions,
    native_functions::{NativeContext, NativeFunction, NativeFunctionTable},
};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    values::{GlobalValue, Struct, Value},
};
use moveos_object_runtime::resolved_arg::ResolvedArg;
use moveos_types::{
    moveos_std::object,
    state::{KeyState, MoveState},
};
use moveos_types::{moveos_std::object_id::ObjectID, state_resolver::StateResolver};
use parking_lot::RwLock;
use smallvec::smallvec;
use smt::SPARSE_MERKLE_PLACEHOLDER_HASH;
use std::{
    collections::{btree_map::Entry, BTreeMap, BTreeSet, VecDeque},
    sync::Arc,
};

/// The native table context extension. This needs to be attached to the NativeContextExtensions
/// value which is passed into session functions, so its accessible from natives of this
/// extension.
#[derive(Tid)]
pub struct NativeTableContext<'a> {
    resolver: &'a dyn StateResolver,
    table_data: Arc<RwLock<TableData>>,
}

/// Ensure the error codes in this file is consistent with the error code in raw_table.move
const E_ALREADY_EXISTS: u64 = 1;
const E_NOT_FOUND: u64 = 2;
const _E_DUPLICATE_OPERATION: u64 = 3;
const _E_NOT_EMPTY: u64 = 4; // This is not used, just used to keep consistent with raw_table.move
const _E_TABLE_ALREADY_EXISTS: u64 = 5;

// ===========================================================================================
// Private Data Structures and Constants

//TODO change to ObjectRuntime and migrate to moveos-object-runtime crate
/// A structure representing mutable data of the NativeTableContext. This is in a RefCell
/// of the overall context so we can mutate while still accessing the overall context.
#[derive(Default)]
pub struct TableData {
    new_tables: BTreeSet<ObjectID>,
    removed_tables: BTreeSet<ObjectID>,
    tables: BTreeMap<ObjectID, Table>,
    object_ref_in_args: BTreeMap<ObjectID, Value>,
    object_reference: BTreeMap<ObjectID, GlobalValue>,
}

/// A structure representing table key.
#[derive(Clone, Ord, PartialOrd, PartialEq, Eq, Debug)]
pub struct TableKey {
    pub key_type: TypeTag,
    pub key: Vec<u8>,
}

impl TableKey {
    pub fn new(key_type: TypeTag, key: Vec<u8>) -> Self {
        Self { key_type, key }
    }
}

/// A structure representing runtime table value.
pub struct TableRuntimeValue {
    /// This is the Layout and TypeTag of the value stored in Box<V>
    /// If the value is GlobalValue::None, the Layout and TypeTag are not known
    value_layout_and_type: Option<(MoveTypeLayout, TypeTag)>,
    /// This is the Box<V> value in MoveVM memory
    /// It can be GlobalValue::None
    box_value: GlobalValue,
}

impl TableRuntimeValue {
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

/// A structure representing a single table.
pub struct Table {
    handle: ObjectID,
    content: BTreeMap<TableKey, TableRuntimeValue>,
    size_increment: i64,
}

// =========================================================================================
// Implementation of Native Table Context

impl<'a> NativeTableContext<'a> {
    /// Create a new instance of a native table context. This must be passed in via an
    /// extension into VM session functions.
    pub fn new(resolver: &'a dyn StateResolver, table_data: Arc<RwLock<TableData>>) -> Self {
        Self {
            resolver,
            table_data,
        }
    }

    pub fn table_data(&self) -> Arc<RwLock<TableData>> {
        self.table_data.clone()
    }
}

impl TableData {
    /// Gets or creates a new table in the TableData. This initializes information about
    /// the table, like the type layout for keys and values.
    pub fn get_or_create_table(
        &mut self,
        // _context: &NativeContext,
        handle: ObjectID,
    ) -> PartialVMResult<&mut Table> {
        match self.tables.entry(handle) {
            Entry::Vacant(e) => {
                let table = Table {
                    handle,
                    content: Default::default(),
                    size_increment: 0,
                };
                if log::log_enabled!(log::Level::Trace) {
                    log::trace!("[RawTable] creating table {}", handle);
                }
                Ok(e.insert(table))
            }
            Entry::Occupied(e) => Ok(e.into_mut()),
        }
    }

    pub fn borrow_table(&self, handle: &ObjectID) -> PartialVMResult<&Table> {
        self.tables
            .get(handle)
            .ok_or_else(|| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }

    pub fn exist_table(&self, handle: &ObjectID) -> bool {
        self.tables.contains_key(handle)
    }

    pub fn load_object(&mut self, object_id: &ObjectID) -> VMResult<()> {
        self.object_reference.entry(*object_id).or_insert_with(|| {
            //TODO we should load the ObjectEntity<T> from the resolver
            //Then cache the Object<T>
            let object_id_value = object_id.to_runtime_value();
            GlobalValue::cached(Value::struct_(Struct::pack(vec![object_id_value])))
                .expect("Failed to cache the Struct")
        });
        Ok(())
    }

    pub fn borrow_object(&mut self, object_id: &ObjectID) -> VMResult<Value> {
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
                let ref_value = self.borrow_object(object_id)?;
                //We cache the object reference in the object_ref_in_args
                //Ensure the reference count and the object can not be borrowed in Move
                self.object_ref_in_args.insert(*object_id, ref_value);
            }
        }
        Ok(())
    }

    /// into inner
    pub fn into_inner(
        self,
    ) -> (
        BTreeSet<ObjectID>,
        BTreeSet<ObjectID>,
        BTreeMap<ObjectID, Table>,
    ) {
        let TableData {
            new_tables,
            removed_tables,
            tables,
            object_reference: _,
            object_ref_in_args: _,
        } = self;
        (new_tables, removed_tables, tables)
    }
}

impl Table {
    fn get_or_create_global_value(
        &mut self,
        native_context: &NativeContext,
        table_context: &NativeTableContext,
        key: TableKey,
    ) -> PartialVMResult<(&mut TableRuntimeValue, Option<Option<NumBytes>>)> {
        Ok(match self.content.entry(key.clone()) {
            Entry::Vacant(entry) => {
                let (tv, loaded) = match table_context
                    .resolver
                    .resolve_table_item(
                        &self.handle,
                        &KeyState::new(key.clone().key, key.clone().key_type),
                    )
                    .map_err(|err| {
                        partial_extension_error(format!("remote table resolver failure: {}", err))
                    })? {
                    Some(value_box) => {
                        let value_layout = get_type_layout(native_context, &value_box.value_type)?;
                        let val = deserialize_and_box(&value_layout, &value_box.value)?;
                        (
                            TableRuntimeValue::new(
                                value_layout,
                                value_box.value_type,
                                GlobalValue::cached(val)?,
                            ),
                            Some(NumBytes::new(value_box.value.len() as u64)),
                        )
                    }
                    None => (TableRuntimeValue::none(), None),
                };
                (entry.insert(tv), Some(loaded))
            }
            Entry::Occupied(entry) => (entry.into_mut(), None),
        })
    }

    pub fn get_or_create_global_value_with_layout_fn(
        &mut self,
        resolver: &dyn StateResolver,
        key: TableKey,
        f: impl FnOnce(&TypeTag) -> PartialVMResult<MoveTypeLayout>,
    ) -> PartialVMResult<(&mut TableRuntimeValue, Option<Option<NumBytes>>)> {
        Ok(match self.content.entry(key.clone()) {
            Entry::Vacant(entry) => {
                let (tv, loaded) = match resolver
                    .resolve_table_item(
                        &self.handle,
                        &KeyState::new(key.key.clone(), key.key_type.clone()),
                    )
                    .map_err(|err| {
                        partial_extension_error(format!("remote table resolver failure: {}", err))
                    })? {
                    Some(value_box) => {
                        let value_layout = f(&value_box.value_type)?;
                        let val = deserialize_and_box(&value_layout, &value_box.value)?;
                        (
                            TableRuntimeValue::new(
                                value_layout,
                                value_box.value_type,
                                GlobalValue::cached(val)?,
                            ),
                            Some(NumBytes::new(value_box.value.len() as u64)),
                        )
                    }
                    None => (TableRuntimeValue::none(), None),
                };
                (entry.insert(tv), Some(loaded))
            }
            Entry::Occupied(entry) => (entry.into_mut(), None),
        })
    }

    pub fn get_global_value(&self, key: &TableKey) -> Option<&TableRuntimeValue> {
        self.content.get(key)
    }

    pub fn contains_key(&self, key: &TableKey) -> bool {
        self.content.contains_key(key)
    }

    pub fn into_inner(self) -> (ObjectID, BTreeMap<TableKey, TableRuntimeValue>, i64) {
        let Table {
            handle,
            content,
            size_increment,
        } = self;
        (handle, content, size_increment)
    }
}

// =========================================================================================
// Native Function Implementations

/// Returns all natives for tables.
pub fn table_natives(table_addr: AccountAddress, gas_params: GasParameters) -> NativeFunctionTable {
    let natives: [(&str, &str, NativeFunction); 8] = [
        (
            "raw_table",
            "new_table",
            make_native_new_table(gas_params.common.clone(), gas_params.new_table),
        ),
        (
            "raw_table",
            "add_box",
            make_native_add_box(gas_params.common.clone(), gas_params.add_box),
        ),
        (
            "raw_table",
            "borrow_box",
            make_native_borrow_box(gas_params.common.clone(), gas_params.borrow_box.clone()),
        ),
        (
            "raw_table",
            "borrow_box_mut",
            make_native_borrow_box(gas_params.common.clone(), gas_params.borrow_box),
        ),
        (
            "raw_table",
            "remove_box",
            make_native_remove_box(gas_params.common.clone(), gas_params.remove_box),
        ),
        (
            "raw_table",
            "contains_box",
            make_native_contains_box(gas_params.common, gas_params.contains_box),
        ),
        (
            "raw_table",
            "drop_unchecked_box",
            make_native_drop_unchecked_box(gas_params.drop_unchecked_box),
        ),
        (
            "raw_table",
            "box_length",
            make_native_box_length(gas_params.box_length),
        ),
    ];

    native_functions::make_table_from_iter(table_addr, natives)
}

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
pub struct NewTableGasParameters {
    pub base: InternalGas,
    pub per_byte_in_str: InternalGasPerByte,
}

fn native_new_table(
    _common_gas_params: &CommonGasParameters,
    gas_params: &NewTableGasParameters,
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(args.len(), 1);

    let table_context = context.extensions().get::<NativeTableContext>();
    let mut table_data = table_context.table_data.write();

    let mut cost = gas_params.base;
    let handle = get_table_handle(&mut args)?;
    cost += gas_params.per_byte_in_str * NumBytes::new(handle.to_bytes().len() as u64);
    let table = table_data.get_or_create_table(handle)?;

    // New table's state_root should be the place holder hash.
    let state_root = AccountAddress::new((*SPARSE_MERKLE_PLACEHOLDER_HASH).into());
    // Represent table info
    let table_info_value = Struct::pack(vec![
        Value::address(state_root),
        Value::u64(table.size_increment as u64),
    ]);
    Ok(NativeResult::ok(
        cost,
        smallvec![Value::struct_(table_info_value)],
    ))
}

pub fn make_native_new_table(
    common_gas_params: CommonGasParameters,
    gas_params: NewTableGasParameters,
) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_new_table(&common_gas_params, &gas_params, context, ty_args, args)
        },
    )
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

    let table_context = context.extensions().get::<NativeTableContext>();
    let mut table_data = table_context.table_data.write();

    let val = args.pop_back().unwrap();
    let key = args.pop_back().unwrap();
    let handle = get_table_handle(&mut args)?;

    let mut cost = gas_params.base;

    let table = table_data.get_or_create_table(handle)?;

    let key_layout = type_to_type_layout(context, &ty_args[0])?;
    let key_type = type_to_type_tag(context, &ty_args[0])?;
    let key_bytes = serialize(&key_layout, &key)?;
    let table_key = TableKey::new(key_type, key_bytes.clone());
    cost += gas_params.per_byte_serialized * NumBytes::new(key_bytes.len() as u64);

    let (tv, loaded) = table.get_or_create_global_value(context, table_context, table_key)?;
    cost += common_gas_params.calculate_load_cost(loaded);
    let value_layout = type_to_type_layout(context, &ty_args[1])?;
    let value_type = type_to_type_tag(context, &ty_args[1])?;
    match tv.move_to(val, value_layout, value_type) {
        Ok(_) => {
            table.size_increment += 1;
            Ok(NativeResult::ok(cost, smallvec![]))
        }
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

    let table_context = context.extensions().get::<NativeTableContext>();
    let mut table_data = table_context.table_data.write();

    let key = args.pop_back().unwrap();
    let handle = get_table_handle(&mut args)?;

    let key_layout = type_to_type_layout(context, &ty_args[0])?;
    let key_type = type_to_type_tag(context, &ty_args[0])?;
    let table = table_data.get_or_create_table(handle)?;

    let mut cost = gas_params.base;

    let key_bytes = serialize(&key_layout, &key)?;
    cost += gas_params.per_byte_serialized * NumBytes::new(key_bytes.len() as u64);

    let table_key = TableKey::new(key_type, key_bytes);
    let (tv, loaded) = table.get_or_create_global_value(context, table_context, table_key)?;
    cost += common_gas_params.calculate_load_cost(loaded);
    let value_type = type_to_type_tag(context, &ty_args[1])?;
    match tv.borrow_global(value_type) {
        Ok(ref_val) => Ok(NativeResult::ok(cost, smallvec![ref_val])),
        Err(_) => Ok(NativeResult::err(cost, E_NOT_FOUND)),
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

    let table_context = context.extensions().get::<NativeTableContext>();
    let mut table_data = table_context.table_data.write();

    let key = args.pop_back().unwrap();
    let handle = get_table_handle(&mut args)?;

    let key_layout = type_to_type_layout(context, &ty_args[0])?;
    let key_type = type_to_type_tag(context, &ty_args[0])?;
    let table = table_data.get_or_create_table(handle)?;

    let mut cost = gas_params.base;
    if log::log_enabled!(log::Level::Trace) {
        log::trace!(
            "[RawTable] contains: table_handle: {}, key_type: {}",
            &&table.handle,
            key_type
        );
    }

    let key_bytes = serialize(&key_layout, &key)?;
    cost += gas_params.per_byte_serialized * NumBytes::new(key_bytes.len() as u64);

    let table_key = TableKey::new(key_type, key_bytes);
    let (tv, loaded) = table.get_or_create_global_value(context, table_context, table_key)?;
    cost += common_gas_params.calculate_load_cost(loaded);

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

    let table_context = context.extensions().get::<NativeTableContext>();
    let mut table_data = table_context.table_data.write();

    let key = args.pop_back().unwrap();
    let handle = get_table_handle(&mut args)?;

    let key_layout = type_to_type_layout(context, &ty_args[0])?;
    let key_type = type_to_type_tag(context, &ty_args[0])?;
    let table = table_data.get_or_create_table(handle)?;

    let mut cost = gas_params.base;

    let key_bytes = serialize(&key_layout, &key)?;
    cost += gas_params.per_byte_serialized * NumBytes::new(key_bytes.len() as u64);

    let table_key = TableKey::new(key_type, key_bytes);
    let (tv, loaded) = table.get_or_create_global_value(context, table_context, table_key)?;
    cost += common_gas_params.calculate_load_cost(loaded);
    let value_type = type_to_type_tag(context, &ty_args[1])?;
    match tv.move_from(value_type) {
        Ok(val) => {
            table.size_increment -= 1;
            Ok(NativeResult::ok(cost, smallvec![val]))
        }
        Err(_) => Ok(NativeResult::err(cost, E_NOT_FOUND)),
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

#[derive(Debug, Clone)]
pub struct BoxLengthGasParameters {
    pub base: InternalGas,
}

fn native_box_length(
    gas_params: &BoxLengthGasParameters,
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(args.len(), 1);

    let table_context = context.extensions().get::<NativeTableContext>();
    let table_data = table_context.table_data.write();

    let handle = get_table_handle(&mut args)?;

    let remote_table_size = table_context
        .resolver
        .resolve_object_state(&handle)
        .map_err(|err| partial_extension_error(format!("remote table resolver failure: {}", err)))?
        .map(|state| state.as_raw_object())
        .transpose()
        .map_err(|err| partial_extension_error(format!("remote table resolver failure: {}", err)))?
        .map_or_else(|| 0u64, |obj| obj.size);

    let size_increment = if table_data.exist_table(&handle) {
        table_data.borrow_table(&handle).unwrap().size_increment
    } else {
        0i64
    };
    let updated_table_size = (remote_table_size as i64) + size_increment;
    debug_assert!(updated_table_size >= 0);

    let length = Value::u64(updated_table_size as u64);
    let cost = gas_params.base;

    Ok(NativeResult::ok(cost, smallvec![length]))
}

pub fn make_native_box_length(gas_params: BoxLengthGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_box_length(&gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct DropUncheckedBoxGasParameters {
    pub base: InternalGas,
}

fn native_drop_unchecked_box(
    gas_params: &DropUncheckedBoxGasParameters,
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(args.len(), 1);

    let table_context = context.extensions().get::<NativeTableContext>();
    let mut table_data = table_context.table_data.write();

    let handle = get_table_handle(&mut args)?;
    table_data.tables.remove(&handle);

    table_data.removed_tables.insert(handle);
    Ok(NativeResult::ok(gas_params.base, smallvec![]))
}

pub fn make_native_drop_unchecked_box(gas_params: DropUncheckedBoxGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_drop_unchecked_box(&gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub common: CommonGasParameters,
    pub new_table: NewTableGasParameters,
    pub add_box: AddBoxGasParameters,
    pub borrow_box: BorrowBoxGasParameters,
    pub contains_box: ContainsBoxGasParameters,
    pub remove_box: RemoveGasParameters,
    pub drop_unchecked_box: DropUncheckedBoxGasParameters,
    pub box_length: BoxLengthGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            common: CommonGasParameters {
                load_base: 0.into(),
                load_per_byte: 0.into(),
                load_failure: 0.into(),
            },
            new_table: NewTableGasParameters {
                base: 0.into(),
                per_byte_in_str: 0.into(),
            },
            add_box: AddBoxGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            borrow_box: BorrowBoxGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            contains_box: ContainsBoxGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            remove_box: RemoveGasParameters {
                base: 0.into(),
                per_byte_serialized: 0.into(),
            },
            drop_unchecked_box: DropUncheckedBoxGasParameters { base: 0.into() },
            box_length: BoxLengthGasParameters { base: 0.into() },
        }
    }
}

// =========================================================================================
// Helpers

fn get_table_handle(args: &mut VecDeque<Value>) -> PartialVMResult<ObjectID> {
    let handle = args.pop_back().unwrap();
    ObjectID::from_runtime_value(handle).map_err(|e| {
        PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(e.to_string())
    })
}

pub fn serialize(layout: &MoveTypeLayout, val: &Value) -> PartialVMResult<Vec<u8>> {
    val.simple_serialize(layout).ok_or_else(|| {
        partial_extension_error(format!(
            "cannot serialize table key or value, layout:{:?}, val:{:?}",
            layout, val
        ))
    })
}

// Deserialize a value and box it to `moveos_std::raw_table::Box<V>`.
fn deserialize_and_box(layout: &MoveTypeLayout, bytes: &[u8]) -> PartialVMResult<Value> {
    let value = Value::simple_deserialize(bytes, layout).ok_or_else(|| {
        partial_extension_error(format!(
            "cannot deserialize table key or value, layout:{:?}, bytes:{:?}",
            layout,
            hex::encode(bytes)
        ))
    })?;
    Ok(Value::struct_(Struct::pack(vec![value])))
}

fn partial_extension_error(msg: impl ToString) -> PartialVMError {
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
