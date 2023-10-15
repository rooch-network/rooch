// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use better_any::{Tid, TidAble};
use move_binary_format::{
    compatibility::Compatibility,
    errors::{PartialVMError, PartialVMResult},
    normalized, CompiledModule,
};
use move_core_types::{
    account_address::AccountAddress,
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    identifier::Identifier,
    language_storage::ModuleId,
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
use moveos_stdlib_builder::dependency_order::sort_by_dependency_order;
use smallvec::smallvec;
use std::collections::{BTreeSet, VecDeque};

// ========================================================================================

const E_ADDRESS_NOT_MATCH_WITH_SIGNER: u64 = 1;
const E_MODULE_VERIFICATION_ERROR: u64 = 2;
const E_MODULE_INCOMPATIBLE: u64 = 3;

/// The native module context.
#[derive(Tid)]
pub struct NativeModuleContext<'a> {
    resolver: &'a dyn ModuleResolver<Error = anyhow::Error>,
    pub init_functions: BTreeSet<ModuleId>,
}

impl<'a> NativeModuleContext<'a> {
    /// Create a new instance of a native table context. This must be passed in via an
    /// extension into VM session functions.
    pub fn new(resolver: &'a dyn ModuleResolver<Error = anyhow::Error>) -> Self {
        Self {
            resolver,
            init_functions: BTreeSet::new(),
        }
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
            gas_params.per_byte_in_str * NumBytes::new(name.len() as u64)
        } else {
            0.into()
        };
    let output = Struct::pack(vec![Value::vector_u8(name.as_bytes().to_vec())]);
    let output_value = Value::struct_(output);
    Ok(NativeResult::ok(cost, smallvec![output_value]))
}

/***************************************************************************************************
 * native fun sort_and_verify_modules_inner(
 *      modules: &vector<vector<u8>>,
 *      account_address: address
 * ): (vector<String>, vector<String>);
 * Return
 *  The first vector is the module names of all the modules.
 *  The second vector is the module names of the modules with init function.
 **************************************************************************************************/

#[derive(Clone, Debug)]
pub struct VerifyModulesGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

fn native_sort_and_verify_modules_inner(
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
    let compiled_modules = sort_by_dependency_order(&compiled_modules).map_err(|e| {
        PartialVMError::new(StatusCode::CYCLIC_MODULE_DEPENDENCY).with_message(e.to_string())
    })?;
    // move verifier
    context.verify_module_bundle_for_publication(&compiled_modules)?;

    // moveos verifier
    let module_context = context.extensions_mut().get_mut::<NativeModuleContext>();
    let mut module_names = vec![];
    let mut init_identifier = vec![];
    for module in &compiled_modules {
        if *module.self_id().address() != account_address {
            return Ok(NativeResult::err(
                cost,
                moveos_types::move_std::error::invalid_argument(E_ADDRESS_NOT_MATCH_WITH_SIGNER),
            ));
        }
        let result = moveos_verifier::verifier::verify_module(module, module_context.resolver);
        match result {
            Ok(res) => {
                if res {
                    init_identifier.push(module.self_id());
                    module_names.push(module.self_id().name().to_owned().into_string());
                } else {
                    return Ok(NativeResult::err(
                        cost,
                        moveos_types::move_std::error::invalid_argument(
                            E_MODULE_VERIFICATION_ERROR,
                        ),
                    ));
                }
            }
            Err(_) => return Err(PartialVMError::new(StatusCode::VERIFICATION_ERROR)),
        }
    }

    let module_names: Vec<Value> = module_names
        .iter()
        .map(|name| {
            Value::struct_(Struct::pack(vec![Value::vector_u8(
                name.as_bytes().to_vec(),
            )]))
        })
        .collect();
    let module_names = Vector::pack(&Type::Struct(CachedStructIndex(0)), module_names)?;

    let init_module_names: Vec<Value> = init_identifier
        .iter()
        .map(|id| id.name().to_owned().into_string())
        .map(|name| {
            Value::struct_(Struct::pack(vec![Value::vector_u8(
                name.as_bytes().to_vec(),
            )]))
        })
        .collect();
    let init_module_names = Vector::pack(&Type::Struct(CachedStructIndex(0)), init_module_names)?;
    Ok(NativeResult::ok(
        cost,
        smallvec![module_names, init_module_names],
    ))
}

/***************************************************************************************************
 * native fun request_init_functions(
 *      module_names: vector<String>,
 *      account_address: address
 * );
 * module_names: names of modules which have a init function
 * account_address: address of all the modules
 **************************************************************************************************/

#[derive(Clone, Debug)]
pub struct RequestInitFunctionsGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

fn request_init_functions(
    gas_params: &RequestInitFunctionsGasParameters,
    context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let mut cost = gas_params.base;
    let account_address = pop_arg!(args, AccountAddress);
    let module_context = context.extensions_mut().get_mut::<NativeModuleContext>();
    for name_str in pop_arg!(args, Vec<Value>) {
        let mut fields = name_str.value_as::<Struct>()?.unpack()?; // std::string::String;
        let val = fields.next().ok_or_else(|| {
            PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE)
                .with_message("There must have only one field".to_owned())
        })?;
        let name_bytes = val.value_as::<Vec<u8>>()?;
        cost += gas_params.per_byte * NumBytes::new(name_bytes.len() as u64);
        let module_id = ModuleId::new(
            account_address,
            Identifier::from_utf8(name_bytes).map_err(|e| {
                PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(e.to_string())
            })?,
        );
        module_context.init_functions.insert(module_id);
    }
    Ok(NativeResult::ok(cost, smallvec![]))
}

/***************************************************************************************************
 * native fun check_compatibililty_inner(
 *      new_bytecodes: vector<u8>,
 *      old_bytecodes: vector<u8>
 * );
 * Check module compatibility when upgrading,
 * Abort if the new module is not compatible with the old module.
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct CheckCompatibilityInnerGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

fn check_compatibililty_inner(
    gas_params: &CheckCompatibilityInnerGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    let mut cost = gas_params.base;
    // TODO: config compatibility through global configuration
    let compat = Compatibility::full_check();
    if compat.need_check_compat() {
        let old_bytecodes = pop_arg!(args, Vec<u8>);
        let new_bytecodes = pop_arg!(args, Vec<u8>);
        cost += gas_params.per_byte * NumBytes::new(new_bytecodes.len() as u64);
        cost += gas_params.per_byte * NumBytes::new(old_bytecodes.len() as u64);
        let new_module = CompiledModule::deserialize(&new_bytecodes)?;
        let old_module = CompiledModule::deserialize(&old_bytecodes)?;
        let new_m = normalized::Module::new(&new_module);
        let old_m = normalized::Module::new(&old_module);

        match compat.check(&old_m, &new_m) {
            Ok(_) => {}
            Err(_) => {
                return Ok(NativeResult::err(
                    cost,
                    moveos_types::move_std::error::invalid_argument(E_MODULE_INCOMPATIBLE),
                ))
            }
        }
    }
    Ok(NativeResult::ok(cost, smallvec![]))
}
/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub module_name_inner: ModuleNameInnerGasParameters,
    pub sort_and_verify_modules_inner: VerifyModulesGasParameters,
    pub request_init_functions: RequestInitFunctionsGasParameters,
    pub check_compatibililty_inner: CheckCompatibilityInnerGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            module_name_inner: ModuleNameInnerGasParameters {
                base: 0.into(),
                per_byte_in_str: 0.into(),
            },
            sort_and_verify_modules_inner: VerifyModulesGasParameters {
                base: 0.into(),
                per_byte: 0.into(),
            },
            request_init_functions: RequestInitFunctionsGasParameters {
                base: 0.into(),
                per_byte: 0.into(),
            },
            check_compatibililty_inner: CheckCompatibilityInnerGasParameters {
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
            "sort_and_verify_modules_inner",
            make_native(
                gas_params.sort_and_verify_modules_inner,
                native_sort_and_verify_modules_inner,
            ),
        ),
        (
            "request_init_functions",
            make_native(gas_params.request_init_functions, request_init_functions),
        ),
        (
            "check_compatibililty_inner",
            make_native(
                gas_params.check_compatibililty_inner,
                check_compatibililty_inner,
            ),
        ),
    ];
    make_module_natives(natives)
}
