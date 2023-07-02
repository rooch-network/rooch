// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use better_any::{Tid, TidAble};
use move_binary_format::{
    errors::{PartialVMError, PartialVMResult},
    CompiledModule,
};
use move_core_types::{
    account_address::AccountAddress,
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    resolver::ModuleResolver,
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::{CachedStructIndex, Type},
    natives::function::NativeResult,
    pop_arg,
    values::{Struct, Value, Vector, VectorRef},
};
use smallvec::smallvec;
use std::collections::VecDeque;

// ========================================================================================

/// The native module context.
#[derive(Tid)]
pub struct NativeModuleContext<'a> {
    resolver: &'a dyn ModuleResolver<Error = anyhow::Error>,
}

impl<'a> NativeModuleContext<'a> {
    /// Create a new instance of a native table context. This must be passed in via an
    /// extension into VM session functions.
    pub fn new(resolver: &'a dyn ModuleResolver<Error = anyhow::Error>) -> Self {
        Self { resolver }
    }
}

/***************************************************************************************************
 * native fun module_name_inner(byte_codes: &vector<u8>): String;
 **************************************************************************************************/
#[derive(Clone, Debug)]
pub struct ModuleNameInnerGasParameters {
    pub base: InternalGas,
    pub per_byte_in_str: InternalGasPerByte,
}

fn native_module_name_inner(
    gas_params: &ModuleNameInnerGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let byte_codes = pop_arg!(args, VectorRef);
    let byte_codes_ref = byte_codes.as_bytes_ref();

    let module = CompiledModule::deserialize(&byte_codes_ref)?;
    let name = module.self_id().name().to_owned().into_string();
    let cost = gas_params.base
        + if gas_params.per_byte_in_str > 0.into() {
            //TODO charge gas cost
            gas_params.per_byte_in_str * NumBytes::new(name.len() as u64)
        } else {
            0.into()
        };
    let output = Struct::pack(vec![Value::vector_u8(name.as_bytes().to_vec())]);
    let output_value = Value::struct_(output);
    Ok(NativeResult::ok(cost, smallvec![output_value]))
}

/***************************************************************************************************
 * native fun verify_modules_inner(
 *      modules: &vector<vector<u8>>,
 *      account_address: address
 * ): vector<String>;
 **************************************************************************************************/

#[derive(Clone, Debug)]
pub struct VerifyModulesGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

fn native_verify_modules_inner(
    gas_params: &VerifyModulesGasParameters,
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let mut cost = gas_params.base;
    let account_address = pop_arg!(args, AccountAddress);
    let mut bundle = vec![];
    for module in pop_arg!(args, Vec<Value>) {
        let byte_codes = module.value_as::<Vec<u8>>()?;
        cost += gas_params.per_byte * NumBytes::new(byte_codes.len() as u64);
        bundle.push(byte_codes);
    }
    let compiled_modules = bundle
        .iter()
        .map(|b| CompiledModule::deserialize(b))
        .collect::<PartialVMResult<Vec<CompiledModule>>>()?;
    let module_context = context.extensions().get::<NativeModuleContext>();
    let mut init_function_modules = vec![];
    let mut module_names = vec![];
    for module in &compiled_modules {
        let result = moveos_verifier::verifier::verify_module(module, module_context.resolver);
        match result {
            Ok(res) => {
                if res {
                    init_function_modules.push(module.self_id());
                    module_names.push(module.self_id().name().to_owned().into_string());
                }
            }
            Err(_) => return Err(PartialVMError::new(StatusCode::VERIFICATION_ERROR)),
        }
    }

    // TODO: handle the init_functions

    let module_names: Vec<Value> = module_names
        .iter()
        .map(|name| {
            let name_struct = Struct::pack(vec![Value::vector_u8(name.as_bytes().to_vec())]);
            Value::struct_(name_struct)
        })
        .collect();
    let output = Vector::pack(&Type::Struct(CachedStructIndex(0)), module_names)?;
    Ok(NativeResult::ok(cost, smallvec![output]))
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub module_name_inner: ModuleNameInnerGasParameters,
    pub verify_modules_inner: VerifyModulesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            module_name_inner: ModuleNameInnerGasParameters {
                base: 0.into(),
                per_byte_in_str: 0.into(),
            },
            verify_modules_inner: VerifyModulesGasParameters {
                base: 0.into(),
                per_byte: 0.into(),
            },
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        (
            "module_name_inner",
            make_native(gas_params.module_name_inner, native_module_name_inner),
        ),
        (
            "verify_modules_inner",
            make_native(gas_params.verify_modules_inner, native_verify_modules_inner),
        ),
    ];
    make_module_natives(natives)
}
