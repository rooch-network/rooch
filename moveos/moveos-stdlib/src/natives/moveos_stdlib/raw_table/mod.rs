// Copyright (c) RoochNetwork
// Copyright (c) The Diem Core Contributors
// Copyright (c) The Move Contributors
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers;
/// A native Table implementation for save any type of value.
/// Refactor from https://github.com/rooch-network/move/blob/c7d8c2b0cdd06dbd90e0ab306932356620b5648a/language/extensions/move-table-extension/src/lib.rs#L4
use better_any::{Tid, TidAble};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
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
    pop_arg,
    values::{GlobalValue, Struct, StructRef, Value},
};
use moveos_types::{object::ObjectID, state::TableTypeInfo, state_resolver::StateResolver};
use parking_lot::RwLock;
use smallvec::smallvec;
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
    //tx_hash: [u8; 32],
    table_data: Arc<RwLock<TableData>>,
}

/// Ensure the error codes in this file is consistent with the error code in raw_table.move
const E_ALREADY_EXISTS: u64 = 1;
const E_NOT_FOUND: u64 = 2;
const E_NOT_EMPTY: u64 = 3;

// ===========================================================================================
// Private Data Structures and Constants

/// A structure representing mutable data of the NativeTableContext. This is in a RefCell
/// of the overall context so we can mutate while still accessing the overall context.
#[derive(Default)]
pub struct TableData {
    new_tables: BTreeMap<ObjectID, TableTypeInfo>,
    removed_tables: BTreeSet<ObjectID>,
    tables: BTreeMap<ObjectID, Table>,
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
    key_layout: MoveTypeLayout,
    content: BTreeMap<Vec<u8>, TableRuntimeValue>,
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
}

impl TableData {
    /// Gets or creates a new table in the TableData. This initializes information about
    /// the table, like the type layout for keys and values.
    fn get_or_create_table(
        &mut self,
        context: &NativeContext,
        handle: ObjectID,
        key_ty: &Type,
    ) -> PartialVMResult<&mut Table> {
        Ok(match self.tables.entry(handle) {
            Entry::Vacant(e) => {
                let key_layout = type_to_type_layout(context, key_ty)?;
                let table = Table {
                    handle,
                    key_layout,
                    content: Default::default(),
                };
                if log::log_enabled!(log::Level::Trace) {
                    let key_type = type_to_type_tag(context, key_ty)?;
                    log::trace!("[RawTable] creating table {} with key {}", handle, key_type);
                }
                e.insert(table)
            }
            Entry::Occupied(e) => e.into_mut(),
        })
    }

    pub fn get_or_create_table_with_key_layout(
        &mut self,
        handle: ObjectID,
        key_layout: MoveTypeLayout,
    ) -> PartialVMResult<&mut Table> {
        Ok(match self.tables.entry(handle) {
            Entry::Vacant(e) => {
                let table = Table {
                    handle,
                    key_layout,
                    content: Default::default(),
                };
                e.insert(table)
            }
            Entry::Occupied(e) => e.into_mut(),
        })
    }

    pub fn borrow_table(&self, handle: &ObjectID) -> PartialVMResult<&Table> {
        self.tables
            .get(handle)
            .ok_or_else(|| PartialVMError::new(StatusCode::STORAGE_ERROR))
    }

    pub fn exist_table(&self, handle: &ObjectID) -> bool {
        self.tables.contains_key(handle)
    }

    /// into inner
    pub fn into_inner(
        self,
    ) -> (
        BTreeMap<ObjectID, TableTypeInfo>,
        BTreeSet<ObjectID>,
        BTreeMap<ObjectID, Table>,
    ) {
        let TableData {
            new_tables,
            removed_tables,
            tables,
        } = self;
        (new_tables, removed_tables, tables)
    }
}

impl Table {
    fn get_or_create_global_value(
        &mut self,
        native_context: &NativeContext,
        table_context: &NativeTableContext,
        key: Vec<u8>,
    ) -> PartialVMResult<(&mut TableRuntimeValue, Option<Option<NumBytes>>)> {
        Ok(match self.content.entry(key) {
            Entry::Vacant(entry) => {
                let (tv, loaded) = match table_context
                    .resolver
                    .resolve_state(&self.handle, entry.key())
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
        key: Vec<u8>,
        f: impl FnOnce(&TypeTag) -> PartialVMResult<MoveTypeLayout>,
    ) -> PartialVMResult<(&mut TableRuntimeValue, Option<Option<NumBytes>>)> {
        Ok(match self.content.entry(key) {
            Entry::Vacant(entry) => {
                let (tv, loaded) =
                    match resolver
                        .resolve_state(&self.handle, entry.key())
                        .map_err(|err| {
                            partial_extension_error(format!(
                                "remote table resolver failure: {}",
                                err
                            ))
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

    pub fn get_global_value(&self, key: &Vec<u8>) -> Option<&TableRuntimeValue> {
        self.content.get(key)
    }

    pub fn contains_key(&self, key: &Vec<u8>) -> bool {
        self.content.contains_key(key)
    }

    pub fn into_inner(
        self,
    ) -> (
        ObjectID,
        MoveTypeLayout,
        BTreeMap<Vec<u8>, TableRuntimeValue>,
    ) {
        let Table {
            handle,
            key_layout,
            content,
        } = self;
        (handle, key_layout, content)
    }

    pub fn key_layout(&self) -> &MoveTypeLayout {
        &self.key_layout
    }
}

// =========================================================================================
// Native Function Implementations

/// Returns all natives for tables.
pub fn table_natives(table_addr: AccountAddress, gas_params: GasParameters) -> NativeFunctionTable {
    let natives: [(&str, &str, NativeFunction); 7] = [
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
            "destroy_empty_box",
            make_native_destroy_empty_box(gas_params.destroy_empty_box),
        ),
        (
            "raw_table",
            "drop_unchecked_box",
            make_native_drop_unchecked_box(gas_params.drop_unchecked_box),
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
    let handle = get_table_handle(pop_arg!(args, StructRef))?;

    let mut cost = gas_params.base;

    let table = table_data.get_or_create_table(context, handle, &ty_args[0])?;

    let key_bytes = serialize(&table.key_layout, &key)?;
    cost += gas_params.per_byte_serialized * NumBytes::new(key_bytes.len() as u64);

    let (tv, loaded) = table.get_or_create_global_value(context, table_context, key_bytes)?;
    cost += common_gas_params.calculate_load_cost(loaded);
    let value_layout = type_to_type_layout(context, &ty_args[1])?;
    let value_type = type_to_type_tag(context, &ty_args[1])?;
    match tv.move_to(val, value_layout, value_type) {
        Ok(_) => Ok(NativeResult::ok(cost, smallvec![])),
        Err(_) => Ok(NativeResult::err(
            cost,
            moveos_types::move_std::error::already_exists(E_ALREADY_EXISTS),
        )),
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
    let handle = get_table_handle(pop_arg!(args, StructRef))?;

    let table = table_data.get_or_create_table(context, handle, &ty_args[0])?;

    let mut cost = gas_params.base;

    let key_bytes = serialize(&table.key_layout, &key)?;
    cost += gas_params.per_byte_serialized * NumBytes::new(key_bytes.len() as u64);

    let (tv, loaded) = table.get_or_create_global_value(context, table_context, key_bytes)?;
    cost += common_gas_params.calculate_load_cost(loaded);
    let value_type = type_to_type_tag(context, &ty_args[1])?;
    match tv.borrow_global(value_type) {
        Ok(ref_val) => Ok(NativeResult::ok(cost, smallvec![ref_val])),
        Err(_) => Ok(NativeResult::err(
            cost,
            moveos_types::move_std::error::not_found(E_NOT_FOUND),
        )),
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
    let handle = get_table_handle(pop_arg!(args, StructRef))?;

    let table = table_data.get_or_create_table(context, handle, &ty_args[0])?;

    let mut cost = gas_params.base;

    if log::log_enabled!(log::Level::Trace) {
        log::trace!(
            "[RawTable] contains: table_handle: {}, key_type: {}",
            &&table.handle,
            type_to_type_tag(context, &ty_args[0])?
        );
    }

    let key_bytes = serialize(&table.key_layout, &key)?;
    cost += gas_params.per_byte_serialized * NumBytes::new(key_bytes.len() as u64);

    let (tv, loaded) = table.get_or_create_global_value(context, table_context, key_bytes)?;
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
    let handle = get_table_handle(pop_arg!(args, StructRef))?;

    let table = table_data.get_or_create_table(context, handle, &ty_args[0])?;

    let mut cost = gas_params.base;

    let key_bytes = serialize(&table.key_layout, &key)?;
    cost += gas_params.per_byte_serialized * NumBytes::new(key_bytes.len() as u64);
    let (tv, loaded) = table.get_or_create_global_value(context, table_context, key_bytes)?;
    cost += common_gas_params.calculate_load_cost(loaded);
    let value_type = type_to_type_tag(context, &ty_args[1])?;
    match tv.move_from(value_type) {
        Ok(val) => Ok(NativeResult::ok(cost, smallvec![val])),
        Err(_) => Ok(NativeResult::err(
            cost,
            moveos_types::move_std::error::not_found(E_NOT_FOUND),
        )),
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
pub struct DestroyEmptyBoxGasParameters {
    pub base: InternalGas,
}

fn native_destroy_empty_box(
    gas_params: &DestroyEmptyBoxGasParameters,
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(args.len(), 1);

    let table_context = context.extensions().get::<NativeTableContext>();
    let mut table_data = table_context.table_data.write();

    let handle = get_table_handle(pop_arg!(args, StructRef))?;
    if table_data.tables.contains_key(&handle)
        && !table_data.tables.get(&handle).unwrap().content.is_empty()
    {
        return Ok(NativeResult::err(
            gas_params.base,
            moveos_types::move_std::error::invalid_state(E_NOT_EMPTY),
        ));
    }
    assert!(table_data.removed_tables.insert(handle));

    Ok(NativeResult::ok(gas_params.base, smallvec![]))
}

pub fn make_native_destroy_empty_box(gas_params: DestroyEmptyBoxGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_destroy_empty_box(&gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct DropUncheckedBoxGasParameters {
    pub base: InternalGas,
}

fn native_drop_unchecked_box(
    gas_params: &DropUncheckedBoxGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    assert_eq!(args.len(), 1);
    //TODO remove the droped table from the table_data
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
    pub add_box: AddBoxGasParameters,
    pub borrow_box: BorrowBoxGasParameters,
    pub contains_box: ContainsBoxGasParameters,
    pub remove_box: RemoveGasParameters,
    pub destroy_empty_box: DestroyEmptyBoxGasParameters,
    pub drop_unchecked_box: DropUncheckedBoxGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            common: CommonGasParameters {
                load_base: 0.into(),
                load_per_byte: 0.into(),
                load_failure: 0.into(),
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
            destroy_empty_box: DestroyEmptyBoxGasParameters { base: 0.into() },
            drop_unchecked_box: DropUncheckedBoxGasParameters { base: 0.into() },
        }
    }
}

// =========================================================================================
// Helpers

// The handle type in Move is `&ObjectID`. This function extracts the address from `ObjectID`.
fn get_table_handle(table: StructRef) -> PartialVMResult<ObjectID> {
    helpers::get_object_id(table)
}

pub fn serialize(layout: &MoveTypeLayout, val: &Value) -> PartialVMResult<Vec<u8>> {
    val.simple_serialize(layout).ok_or_else(|| {
        partial_extension_error(format!(
            "cannot serialize table key or value, layout:{:?}, val:{:?}",
            layout, val
        ));
        panic!("debug")
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
