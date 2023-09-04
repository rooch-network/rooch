// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::{collections::VecDeque, sync::Arc};

use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{gas_algebra::InternalGas, language_storage::TypeTag, vm_status::StatusCode};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use smallvec::smallvec;

use crate::natives::helpers::make_module_natives;

#[derive(Debug, Clone)]
pub struct ModuleSignerGasParameters {
    pub base: InternalGas,
}

impl ModuleSignerGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: InternalGas::zero(),
        }
    }
}

/***************************************************************************************************
 * native fun module_signer
 *
 *   gas cost: base_cost
 *
 **************************************************************************************************/
#[inline]
fn native_module_signer(
    gas_params: &ModuleSignerGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(arguments.is_empty());

    let type_tag = context.type_to_type_tag(&ty_args[0])?;

    if let TypeTag::Struct(struct_tag) = type_tag {
        Ok(NativeResult::ok(
            gas_params.base,
            smallvec![Value::signer(struct_tag.address)],
        ))
    } else {
        Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
            .with_message("Expected a struct type, but a non-struct type was provided".to_owned()))
    }
}

pub fn make_native_module_signer(gas_params: ModuleSignerGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| {
        native_module_signer(&gas_params, context, ty_args, args)
    })
}

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub module_signer: ModuleSignerGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            module_signer: ModuleSignerGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [(
        "module_signer",
        make_native_module_signer(gas_params.module_signer),
    )];

    make_module_natives(natives)
}
