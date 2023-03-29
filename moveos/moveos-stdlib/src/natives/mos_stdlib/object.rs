// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright (c) Mysten Labs, Inc.
// SPDX-License-Identifier: Apache-2.0

// Source from https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/src/natives/object.rs
// and do some refactor

use super::object_extension::TransferResult;
use crate::natives::{
    helpers::{make_module_natives, make_native},
    mos_stdlib::object_extension::NativeObjectContext,
    BaseGasParameter,
};
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    account_address::AccountAddress, language_storage::TypeTag, value::MoveTypeLayout,
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{StructRef, Value},
};
use moveos_types::object::Owner;
use smallvec::smallvec;
use std::collections::VecDeque;

pub type BorrowUidGasParameter = BaseGasParameter;

// native fun borrow_uid<T: key>(obj: &T): &UID;
pub fn native_borrow_uid(
    gas_param: &BorrowUidGasParameter,
    _context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    let obj = pop_arg!(args, StructRef);
    let id_field = obj.borrow_field(0)?;
    //TODO check the id_field type.

    // TODO: what should the cost of this be?
    let cost = gas_param.base;

    Ok(NativeResult::ok(cost, smallvec![id_field]))
}

pub type DeleteGasParameter = BaseGasParameter;

// native fun delete_impl(id: address);
pub fn native_delete_impl(
    gas_param: &DeleteGasParameter,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.is_empty());
    debug_assert!(args.len() == 1);

    // unwrap safe because the interface of native function guarantees it.
    let uid_bytes = pop_arg!(args, AccountAddress);

    // TODO: what should the cost of this be?
    let cost = gas_param.base;

    let obj_runtime: &mut NativeObjectContext = context.extensions_mut().get_mut();
    obj_runtime.delete_id(uid_bytes.into())?;
    Ok(NativeResult::ok(cost, smallvec![]))
}

// We make the transfer function with object native function together.
// https://github.com/MystenLabs/sui/blob/598f106ef5fbdfbe1b644236f0caf46c94f4d1b7/crates/sui-framework/src/natives/transfer.rs

pub type TransferGasParameter = BaseGasParameter;

/// Implementation of Move native function
/// `transfer_internal<T: key>(obj: T, recipient: address)`
pub fn native_transfer_internal(
    gas_param: &TransferGasParameter,
    context: &mut NativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 2);

    let ty = ty_args.pop().unwrap();
    let recipient = pop_arg!(args, AccountAddress);
    let obj = args.pop_back().unwrap();
    let owner = Owner::AddressOwner(recipient);
    object_runtime_transfer(context, owner, ty, obj)?;
    // Charge a constant native gas cost here, since
    // we will charge it properly when processing
    // all the events in adapter.
    // TODO: adjust native_gas cost size base.
    let cost = gas_param.base;
    Ok(NativeResult::ok(cost, smallvec![]))
}

pub type FreezeGasParameter = BaseGasParameter;

/// Implementation of Move native function
/// `freeze_object<T: key>(obj: T)`
pub fn native_freeze_object(
    gas_param: &FreezeGasParameter,
    context: &mut NativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    let ty = ty_args.pop().unwrap();
    let obj = args.pop_back().unwrap();
    object_runtime_transfer(context, Owner::Immutable, ty, obj)?;
    let cost = gas_param.base;
    Ok(NativeResult::ok(cost, smallvec![]))
}

pub type ShareGasParameter = BaseGasParameter;

/// Implementation of Move native function
/// `share_object<T: key>(obj: T)`
pub fn native_share_object(
    gas_param: &ShareGasParameter,
    context: &mut NativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    let ty = ty_args.pop().unwrap();
    let obj = args.pop_back().unwrap();
    let _transfer_result = object_runtime_transfer(
        context,
        // Dummy version, to be filled with the correct initial version when the effects of the
        // transaction are written to storage.
        Owner::Shared {
            initial_shared_version: 0,
        },
        ty,
        obj,
    )?;
    let cost = gas_param.base;
    //TODO check the transfer_result
    Ok(NativeResult::ok(cost, smallvec![]))
}

fn object_runtime_transfer(
    context: &mut NativeContext,
    owner: Owner,
    ty: Type,
    obj: Value,
) -> PartialVMResult<TransferResult> {
    let tag = match context.type_to_type_tag(&ty)? {
        TypeTag::Struct(s) => s,
        _ => {
            return Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message("verifier guarantees this is a struct".to_string()),
            )
        }
    };
    let layout = match context.type_to_type_layout(&ty)? {
        Some(MoveTypeLayout::Struct(s)) => s,
        _ => {
            return Err(
                PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR)
                    .with_message("verifier guarantees this is a struct".to_string()),
            )
        }
    };

    let obj_runtime: &mut NativeObjectContext = context.extensions_mut().get_mut();
    obj_runtime.transfer(owner, ty, *tag, layout, obj)
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub borrow_uid: BorrowUidGasParameter,
    pub delete: DeleteGasParameter,
    pub transfer: TransferGasParameter,
    pub freeze: FreezeGasParameter,
    pub share: ShareGasParameter,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            borrow_uid: BorrowUidGasParameter::zeros(),
            delete: DeleteGasParameter::zeros(),
            transfer: TransferGasParameter::zeros(),
            freeze: FreezeGasParameter::zeros(),
            share: ShareGasParameter::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "borrow_uid",
            make_native(gas_params.borrow_uid, native_borrow_uid),
        ),
        (
            "delete_impl",
            make_native(gas_params.delete, native_delete_impl),
        ),
        (
            "transfer_internal",
            make_native(gas_params.transfer, native_transfer_internal),
        ),
        (
            "freeze_object",
            make_native(gas_params.freeze, native_freeze_object),
        ),
        (
            "share_object",
            make_native(gas_params.share, native_share_object),
        ),
    ];

    make_module_natives(natives)
}
