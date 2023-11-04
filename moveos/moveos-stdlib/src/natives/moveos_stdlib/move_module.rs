// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers::{make_module_natives, make_native};
use better_any::{Tid, TidAble};
use itertools::zip_eq;
use move_binary_format::{
    compatibility::Compatibility,
    errors::{PartialVMError, PartialVMResult},
    normalized, CompiledModule,
};
use move_core_types::u256::U256;
use move_core_types::{
    account_address::AccountAddress,
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    identifier::Identifier,
    language_storage::ModuleId,
    resolver::ModuleResolver,
    value::MoveValue,
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
use std::collections::{BTreeSet, HashMap, VecDeque};
use std::hash::Hash;

// ========================================================================================

const E_ADDRESS_NOT_MATCH_WITH_SIGNER: u64 = 1;
const E_MODULE_VERIFICATION_ERROR: u64 = 2;
const E_MODULE_INCOMPATIBLE: u64 = 3;
const E_LENTH_NOT_MATCH: u64 = 4;

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
                }
                module_names.push(module.self_id().name().to_owned().into_string());
            }
            Err(e) => {
                //TODO provide a flag to control whether to print debug log.
                log::info!("module {} verification error: {:?}", module.self_id(), e);
                return Ok(NativeResult::err(
                    cost,
                    moveos_types::move_std::error::invalid_argument(E_MODULE_VERIFICATION_ERROR),
                ));
            }
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
        let name_ident = unpack_string_to_identifier(name_str)?;
        cost += gas_params.per_byte * NumBytes::new(1u64);
        let module_id = ModuleId::new(account_address, name_ident);
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

////////////////////// Native functions for replace identifiers //////////////////////

#[derive(Debug, Clone)]
pub struct ModuleModifyGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

fn module_replace_template<C, F, TF>(
    gas_params: &ModuleModifyGasParameters,
    _context: &mut NativeContext,
    _ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
    element_type: Type,
    modify_fn: F,
    type_cast_fn: TF,
) -> PartialVMResult<NativeResult>
where
    C: PartialEq + Clone + Eq + std::fmt::Debug + Hash,
    F: Fn(&mut CompiledModule, &HashMap<C, C>) -> PartialVMResult<()>,
    TF: Fn(Value) -> PartialVMResult<C>,
{
    debug_assert!(args.len() == 3, "Wrong number of arguments");
    let mut cost = gas_params.base;
    let new_vec = pop_arg!(args, Vector);
    let old_vec = pop_arg!(args, Vector);
    let vec_len = new_vec.elem_views().len();
    if vec_len != old_vec.elem_views().len() {
        return Ok(NativeResult::err(
            cost,
            moveos_types::move_std::error::invalid_argument(E_LENTH_NOT_MATCH),
        ));
    };
    let vec_len = vec_len as u64;
    let new_values = new_vec.unpack(&element_type, vec_len)?;
    let old_values = old_vec.unpack(&element_type, vec_len)?;

    let address_mapping: HashMap<C, C> = zip_eq(old_values, new_values)
        .map(|(a, b)| Ok((type_cast_fn(a)?, type_cast_fn(b)?)))
        .collect::<PartialVMResult<_>>()?;

    let mut bundle = vec![];
    for module in pop_arg!(args, Vec<Value>) {
        let byte_codes = module.value_as::<Vec<u8>>()?;
        cost += gas_params.per_byte * NumBytes::new(byte_codes.len() as u64);
        bundle.push(byte_codes);
    }
    let output_modules = modify_modules(bundle, |module| modify_fn(module, &address_mapping))?;
    Ok(NativeResult::ok(cost, smallvec![output_modules]))
}

/***************************************************************************************************
 * native fun replace_address_identifiers(
 *     bytes: vector<vector<u8>>,
 *     old_addresses: vector<address>,
 *     new_addresses: vector<address>,
 * ): vector<vector<u8>>;
 * Native function to remap addresses in module binary where the length of
 * `old_addresses` must equal to that of `new_addresses`.
 **************************************************************************************************/
fn replace_address_identifiers(
    gas_params: &ModuleModifyGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    module_replace_template(
        gas_params,
        context,
        ty_args,
        args,
        Type::Address,
        module_replace_address_identifiers,
        |a| a.value_as::<AccountAddress>(),
    )
}

/***************************************************************************************************
 * native fun replace_addresses_constant(
 *     bytes: vector<vector<u8>>,
 *     old_addresses: vector<address>,
 *     new_addresses: vector<address>,
 * ): (vector<u8>, vector<vector<u8>>);
 * Native function to replace constant addresses in module binary where the length of
 * `old_addresses` must equal to that of `new_addresses`.
 **************************************************************************************************/
fn replace_addresses_constant(
    gas_params: &ModuleModifyGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    module_replace_template(
        gas_params,
        context,
        ty_args,
        args,
        Type::Address,
        module_replace_constant_addresses,
        |a| a.value_as::<AccountAddress>(),
    )
}

/***************************************************************************************************
 * native fun replace_identifiers(
 *     bytes: vector<vector<u8>>,
 *     old_idents: vector<String>,
 *     new_idents: vector<String>,
 * ): vector<vector<u8>>;
 * Native function to replace the name identifier `old_idents` to `new_idents` in module binary.
 **************************************************************************************************/
fn replace_identifiers(
    gas_params: &ModuleModifyGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    module_replace_template(
        gas_params,
        context,
        ty_args,
        args,
        Type::Struct(CachedStructIndex(0)), // std::string::String
        module_replace_identifiers,
        |a| unpack_string_to_identifier(a),
    )
}

/***************************************************************************************************
 * native public(friend) fun replace_bytes_constant(
 *     bytes: vector<vector<u8>>,
 *     old_bytes: vector<vector<u8>>,
 *     new_bytes: vector<vector<u8>>,
 * ): vector<vector<u8>>;
 * Native function to replace the name identifier `old_idents` to `new_idents` in module binary.
 **************************************************************************************************/
fn replace_bytes_constant(
    gas_params: &ModuleModifyGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    module_replace_template(
        gas_params,
        context,
        ty_args,
        args,
        Type::Vector(Box::new(Type::U8)),
        module_replace_constants,
        |a| a.value_as::<Vec<u8>>(),
    )
}
/***************************************************************************************************
 * native public(friend) fun replace_u8_constant(
 *     bytes: vector<vector<u8>>,
 *     old_u8s: vector<u8>,
 *     new_u8s: vector<u8>,
 * ): vector<vector<u8>>;
 * Native function to replace the u8 constant `old_u8s` to `new_u8s` in module binary.
 **************************************************************************************************/
fn replace_u8_constant(
    gas_params: &ModuleModifyGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    module_replace_template(
        gas_params,
        context,
        ty_args,
        args,
        Type::U8,
        module_replace_constants,
        |a| a.value_as::<u8>(),
    )
}

/***************************************************************************************************
 * native public(friend) fun replace_u64_constant(
 *     bytes: vector<vector<u8>>,
 *     old_u64s: vector<u64>,
 *     new_u64s: vector<u64>,
 * ): vector<vector<u8>>;
 * Native function to replace the u64 constant `old_u64s` to `new_u64s` in module binary.
 **************************************************************************************************/
fn replace_u64_constant(
    gas_params: &ModuleModifyGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    module_replace_template(
        gas_params,
        context,
        ty_args,
        args,
        Type::U64,
        module_replace_constants,
        |a| a.value_as::<u64>(),
    )
}

/***************************************************************************************************
 * native public(friend) fun replace_u256_constant(
 *     bytes: vector<vector<u8>>,
 *     old_u256s: vector<u256>,
 *     new_u256s: vector<u256>,
 * ): vector<vector<u8>>;
 * Native function to replace the u256 constant `old_u256s` to `new_u256s` in module binary.
 **************************************************************************************************/
fn replace_u256_constant(
    gas_params: &ModuleModifyGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    module_replace_template(
        gas_params,
        context,
        ty_args,
        args,
        Type::U256,
        module_replace_constants,
        |a| a.value_as::<U256>(),
    )
}

fn modify_modules(
    module_bundles: Vec<Vec<u8>>,
    replace_fn: impl Fn(&mut CompiledModule) -> PartialVMResult<()>,
) -> PartialVMResult<Value> {
    let mut compiled_modules = module_bundles
        .into_iter()
        .map(|b| CompiledModule::deserialize(&b))
        .collect::<PartialVMResult<Vec<CompiledModule>>>()?;

    let mut remapped_bubdles = vec![];
    for module in compiled_modules.iter_mut() {
        replace_fn(module)?;
        let mut binary: Vec<u8> = vec![];
        module.serialize(&mut binary).map_err(|e| {
            PartialVMError::new(StatusCode::VALUE_SERIALIZATION_ERROR).with_message(e.to_string())
        })?;
        let value = Value::vector_u8(binary);
        remapped_bubdles.push(value);
    }
    let output_modules = Vector::pack(&Type::Vector(Box::new(Type::U8)), remapped_bubdles)?;
    Ok(output_modules)
}

fn movevalue_replace_addresses(value: &mut MoveValue, f: &dyn Fn(&mut AccountAddress)) {
    match value {
        MoveValue::Address(addr) => f(addr),
        MoveValue::Vector(vals) => {
            vals.iter_mut()
                .for_each(|val| movevalue_replace_addresses(val, f));
        }
        // TODO: handle constant addresses in Other struct
        _ => {}
    }
}

fn module_replace_identifiers(
    module: &mut CompiledModule,
    identifier_mapping: &HashMap<Identifier, Identifier>,
) -> PartialVMResult<()> {
    for i in 0..module.identifiers.len() {
        if let Some(new_ident) = identifier_mapping.get(&module.identifiers[i]) {
            module.identifiers[i] = Identifier::new(new_ident.to_string()).map_err(|e| {
                PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(e.to_string())
            })?;
        }
    }
    Ok(())
}

fn module_replace_address_identifiers(
    module: &mut CompiledModule,
    address_mapping: &HashMap<AccountAddress, AccountAddress>,
) -> PartialVMResult<()> {
    // replace addresses in address identifiers.
    for addr in module.address_identifiers.iter_mut() {
        if let Some(new_addr) = address_mapping.get(addr) {
            *addr = *new_addr;
        }
    }
    Ok(())
}

fn module_replace_constant_addresses(
    module: &mut CompiledModule,
    address_mapping: &HashMap<AccountAddress, AccountAddress>,
) -> PartialVMResult<()> {
    // replace addresses in constant.
    for constant in module.constant_pool.iter_mut() {
        let mut constant_value = constant.deserialize_constant().ok_or_else(|| {
            PartialVMError::new(StatusCode::VALUE_DESERIALIZATION_ERROR)
                .with_message("cannot deserialize constant".to_string())
        })?;

        movevalue_replace_addresses(&mut constant_value, &|addr| {
            if let Some(new_addr) = address_mapping.get(addr) {
                *addr = *new_addr;
            }
        });

        let bytes = constant_value.simple_serialize().ok_or_else(|| {
            PartialVMError::new(StatusCode::VALUE_SERIALIZATION_ERROR)
                .with_message("cannot serialize constant".to_string())
        })?;
        constant.data = bytes;
    }
    Ok(())
}

pub trait ConstantOps {
    fn deserialize(value: &MoveValue) -> Option<Self>
    where
        Self: Sized;
    fn serialize(self) -> Result<Vec<u8>, StatusCode>;
}

impl ConstantOps for Vec<u8> {
    fn deserialize(value: &MoveValue) -> Option<Self> {
        if let MoveValue::Vector(vals) = value {
            MoveValue::vec_to_vec_u8(vals.clone()).ok()
        } else {
            None
        }
    }

    fn serialize(self) -> Result<Vec<u8>, StatusCode> {
        MoveValue::vector_u8(self)
            .simple_serialize()
            .ok_or(StatusCode::VALUE_SERIALIZATION_ERROR)
    }
}

impl ConstantOps for u8 {
    fn deserialize(value: &MoveValue) -> Option<Self> {
        if let MoveValue::U8(val) = value {
            Some(*val)
        } else {
            None
        }
    }

    fn serialize(self) -> Result<Vec<u8>, StatusCode> {
        MoveValue::U8(self)
            .simple_serialize()
            .ok_or(StatusCode::VALUE_SERIALIZATION_ERROR)
    }
}

impl ConstantOps for u64 {
    fn deserialize(value: &MoveValue) -> Option<Self> {
        if let MoveValue::U64(val) = value {
            Some(*val)
        } else {
            None
        }
    }

    fn serialize(self) -> Result<Vec<u8>, StatusCode> {
        MoveValue::U64(self)
            .simple_serialize()
            .ok_or(StatusCode::VALUE_SERIALIZATION_ERROR)
    }
}

impl ConstantOps for U256 {
    fn deserialize(value: &MoveValue) -> Option<Self> {
        if let MoveValue::U256(val) = value {
            Some(*val)
        } else {
            None
        }
    }

    fn serialize(self) -> Result<Vec<u8>, StatusCode> {
        MoveValue::U256(self)
            .simple_serialize()
            .ok_or(StatusCode::VALUE_SERIALIZATION_ERROR)
    }
}

fn module_replace_constants<T>(
    module: &mut CompiledModule,
    mapping: &HashMap<T, T>,
) -> PartialVMResult<()>
where
    T: ConstantOps + Clone + Hash + Eq,
{
    // replace T type in constants.
    for constant in module.constant_pool.iter_mut() {
        let constant_value = constant.deserialize_constant().ok_or_else(|| {
            PartialVMError::new(StatusCode::VALUE_DESERIALIZATION_ERROR)
                .with_message("cannot deserialize constant".to_string())
        })?;

        if let Some(old_value) = T::deserialize(&constant_value) {
            if let Some(new_value) = mapping.get(&old_value) {
                constant.data = new_value.clone().serialize().map_err(|code| {
                    PartialVMError::new(code).with_message("cannot serialize constant".to_string())
                })?;
            }
        }
    }
    Ok(())
}

/// Unpack input `std::string::String` to identifier.
fn unpack_string_to_identifier(value: Value) -> PartialVMResult<Identifier> {
    let mut fields = value.value_as::<Struct>()?.unpack()?; // std::string::String;
    let val = fields.next().ok_or_else(|| {
        PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE)
            .with_message("There must have only one field".to_owned())
    })?;
    let bytes = val.value_as::<Vec<u8>>()?;
    let ident = Identifier::from_utf8(bytes).map_err(|e| {
        PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(e.to_string())
    })?;
    Ok(ident)
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
    pub replace_address_identifiers: ModuleModifyGasParameters,
    pub replace_addresses_constant: ModuleModifyGasParameters,
    pub replace_identifiers: ModuleModifyGasParameters,
    pub replace_bytes_constant: ModuleModifyGasParameters,
    pub replace_u8_constant: ModuleModifyGasParameters,
    pub replace_u64_constant: ModuleModifyGasParameters,
    pub replace_u256_constant: ModuleModifyGasParameters,
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
            replace_address_identifiers: ModuleModifyGasParameters {
                base: 0.into(),
                per_byte: 0.into(),
            },
            replace_addresses_constant: ModuleModifyGasParameters {
                base: 0.into(),
                per_byte: 0.into(),
            },
            replace_identifiers: ModuleModifyGasParameters {
                base: 0.into(),
                per_byte: 0.into(),
            },
            replace_bytes_constant: ModuleModifyGasParameters {
                base: 0.into(),
                per_byte: 0.into(),
            },
            replace_u8_constant: ModuleModifyGasParameters {
                base: 0.into(),
                per_byte: 0.into(),
            },
            replace_u64_constant: ModuleModifyGasParameters {
                base: 0.into(),
                per_byte: 0.into(),
            },
            replace_u256_constant: ModuleModifyGasParameters {
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
        (
            "replace_address_identifiers",
            make_native(
                gas_params.replace_address_identifiers,
                replace_address_identifiers,
            ),
        ),
        (
            "replace_addresses_constant",
            make_native(
                gas_params.replace_addresses_constant,
                replace_addresses_constant,
            ),
        ),
        (
            "replace_identifiers",
            make_native(gas_params.replace_identifiers, replace_identifiers),
        ),
        (
            "replace_bytes_constant",
            make_native(gas_params.replace_bytes_constant, replace_bytes_constant),
        ),
        (
            "replace_u8_constant",
            make_native(gas_params.replace_u8_constant, replace_u8_constant),
        ),
        (
            "replace_u64_constant",
            make_native(gas_params.replace_u64_constant, replace_u64_constant),
        ),
        (
            "replace_u256_constant",
            make_native(gas_params.replace_u256_constant, replace_u256_constant),
        ),
    ];
    make_module_natives(natives)
}
