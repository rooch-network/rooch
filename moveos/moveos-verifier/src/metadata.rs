// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

#![allow(clippy::redundant_closure)]
#![allow(clippy::map_clone)]

use crate::build::ROOCH_METADATA_KEY;
use crate::verifier::INIT_FN_NAME_IDENTIFIER;
use itertools::Itertools;
use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::errors::{Location, PartialVMError, VMResult};
use move_binary_format::file_format::{
    Bytecode, FunctionInstantiation, SignatureToken, Visibility,
};
use move_binary_format::CompiledModule;
use move_core_types::language_storage::ModuleId;
use move_core_types::metadata::Metadata;
use move_core_types::vm_status::StatusCode;
use move_model::ast::{Attribute, AttributeValue};
use move_model::model::{FunctionEnv, GlobalEnv, Loc, ModuleEnv, StructEnv};
use move_model::ty::PrimitiveType;
use move_model::ty::Type;
use moveos_types::moveos_std::context::Context;
use moveos_types::state::MoveStructType;
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use thiserror::Error;

// This is only used for local integration testing and compiling multiple Move Packages.
// When publishing, use FunctionIndex -> ModuleID to read the module from the DB.
pub static mut GLOBAL_PRIVATE_GENERICS: Lazy<BTreeMap<String, Vec<usize>>> =
    Lazy::new(|| BTreeMap::new());

pub static mut GLOBAL_DATA_STRUCT: Lazy<BTreeMap<String, bool>> = Lazy::new(|| BTreeMap::new());

pub static mut GLOBAL_DATA_STRUCT_FUNC: Lazy<BTreeMap<String, Vec<usize>>> =
    Lazy::new(|| BTreeMap::new());

pub static mut GLOBAL_GAS_FREE_RECORDER: Lazy<BTreeMap<String, Vec<usize>>> =
    Lazy::new(|| BTreeMap::new());

const PRIVATE_GENERICS_ATTRIBUTE: &str = "private_generics";

const GAS_FREE_ATTRIBUTE: &str = "gas_free";
const GAS_FREE_VALIDATE: &str = "gas_validate";
const GAS_FREE_CHARGE_POST: &str = "gas_charge_post";

const DATA_STRUCT_ATTRIBUTE: &str = "data_struct";

/// Enumeration of potentially known attributes
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub struct KnownAttribute {
    kind: u8,
    args: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, Default)]
pub struct GasFreeFunction {
    pub gas_validate: String,
    pub gas_charge_post: String,
}

/// V1 of Aptos specific metadata attached to the metadata section of file_format.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RuntimeModuleMetadataV1 {
    /// Attributes attached to structs.
    pub struct_attributes: BTreeMap<String, Vec<KnownAttribute>>,

    /// Attributes attached to functions, by definition index.
    pub fun_attributes: BTreeMap<String, Vec<KnownAttribute>>,

    /// The correspondence between private generics and their type parameters.
    pub private_generics_indices: BTreeMap<String, Vec<usize>>,

    /// Save information for the gas free function.
    pub gas_free_function_map: BTreeMap<String, GasFreeFunction>,

    /// Save information for the data_struct.
    pub data_struct_map: BTreeMap<String, bool>,

    /// Save information for the data_struct in the Move function.
    pub data_struct_func_map: BTreeMap<String, Vec<usize>>,
}

impl RuntimeModuleMetadataV1 {
    pub fn is_empty(&self) -> bool {
        self.fun_attributes.is_empty()
            && self.struct_attributes.is_empty()
            && self.private_generics_indices.is_empty()
            && self.gas_free_function_map.is_empty()
    }
}

fn find_metadata<'a>(module: &'a CompiledModule, key: &[u8]) -> Option<&'a Metadata> {
    module.metadata.iter().find(|md| md.key == key)
}

/// Extract metadata from a compiled module/
pub fn get_metadata_from_compiled_module(
    module: &CompiledModule,
) -> Option<RuntimeModuleMetadataV1> {
    if let Some(data) = find_metadata(module, ROOCH_METADATA_KEY) {
        bcs::from_bytes::<RuntimeModuleMetadataV1>(&data.value).ok()
    } else {
        None
    }
}

/// Run the extended context checker on target modules in the environment and returns a map
/// from module to extended runtime metadata. Any errors during context checking are reported to
/// `env`. This is invoked after general build succeeds.
pub fn run_extended_checks(env: &GlobalEnv) -> BTreeMap<ModuleId, RuntimeModuleMetadataV1> {
    let mut checker = ExtendedChecker::new(env);
    checker.run();
    checker.output
}

#[derive(Debug)]
struct ExtendedChecker<'a> {
    env: &'a GlobalEnv,
    /// Computed runtime metadata
    output: BTreeMap<ModuleId, RuntimeModuleMetadataV1>,
}

impl<'a> ExtendedChecker<'a> {
    fn new(env: &'a GlobalEnv) -> Self {
        Self {
            env,
            output: BTreeMap::default(),
        }
    }

    fn run(&mut self) {
        for ref module in self.env.get_modules() {
            if module.is_target() {
                self.check_private_generics_functions(module);
                self.check_entry_functions(module);
                self.check_init_module(module);
                self.check_global_storage_access(module);
                self.check_gas_free_function(module);
                self.check_data_struct(module);
            }
        }
    }
}

// ----------------------------------------------------------------------------------
// Private Generic Functions

impl<'a> ExtendedChecker<'a> {
    fn check_private_generics_functions(&mut self, module: &ModuleEnv) {
        // The `type_name_indices` is used to save the private_generics information of the found function to the metadata of the module.
        // The private_generics information of the function is looked up from `GLOBAL_PRIVATE_GENERICS`.
        let mut type_name_indices: BTreeMap<String, Vec<usize>> = BTreeMap::new();
        let mut func_loc_map = BTreeMap::new();

        let compiled_module = module.get_verified_module();
        let view = BinaryIndexedView::Module(compiled_module);

        // Check every function and if a function has the private_generics attribute,
        // ensure that the function name and the types defined in the private_generics attribute match,
        // for example: #[private_generics(T1, T2)].
        for ref fun in module.get_functions() {
            if self.has_attribute(fun, PRIVATE_GENERICS_ATTRIBUTE) {
                let mut func_type_params_name_list = vec![];
                let type_params = fun.get_named_type_parameters();
                for t in type_params {
                    let type_name = self.env.symbol_pool().string(t.0).as_str().to_string();
                    func_type_params_name_list.push(type_name);
                }

                if func_type_params_name_list.is_empty() {
                    self.env
                        .error(&fun.get_loc(), "Function do not has type parameter.");
                }

                let attributes = fun.get_attributes();

                for attr in attributes {
                    if let Attribute::Apply(_, _, types) = attr {
                        if types.is_empty() {
                            self.env.error(
                                &fun.get_loc(),
                                "A type name is needed for private generics.",
                            );
                        }

                        let mut attribute_type_index = vec![];
                        let mut attribute_type_names = vec![];
                        for (idx, type_name) in func_type_params_name_list.iter().enumerate() {
                            let _ = types
                                .iter()
                                .map(|attr| {
                                    if let Attribute::Apply(_, name, _) = attr {
                                        let attribute_type_name = self
                                            .env
                                            .symbol_pool()
                                            .string(*name)
                                            .as_str()
                                            .to_string();

                                        if attribute_type_name == type_name.as_str() {
                                            attribute_type_index.push(idx);
                                            attribute_type_names.push(attribute_type_name);
                                        }
                                    }
                                })
                                .collect::<Vec<_>>();
                        }

                        let _ = types
                            .iter()
                            .map(|attr| {
                                if let Attribute::Apply(_, name, _) = attr {
                                    let attribute_type_name =
                                        self.env.symbol_pool().string(*name).as_str().to_string();
                                    if !attribute_type_names.contains(&attribute_type_name) {
                                        let func_name = self
                                            .env
                                            .symbol_pool()
                                            .string(fun.get_name())
                                            .as_str()
                                            .to_string();

                                        self.env.error(
                                            &fun.get_loc(),
                                            format!(
                                                "type name {:?} not defined in function {:?}",
                                                attribute_type_name, func_name
                                            )
                                            .as_str(),
                                        );
                                    }
                                }
                            })
                            .collect::<Vec<_>>();

                        let module_address = module.self_address().to_hex_literal();
                        let module_name = self
                            .env
                            .symbol_pool()
                            .string(module.get_name().name())
                            .as_str()
                            .to_string();
                        let func_name = self
                            .env
                            .symbol_pool()
                            .string(fun.get_name())
                            .as_str()
                            .to_string();
                        let full_path_func_name =
                            format!("{}::{}::{}", module_address, module_name, func_name);
                        type_name_indices
                            .insert(full_path_func_name.clone(), attribute_type_index.clone());

                        unsafe {
                            GLOBAL_PRIVATE_GENERICS
                                .insert(full_path_func_name, attribute_type_index.clone());
                        }

                        func_loc_map.insert(func_name, fun.get_loc());
                    }
                }
            }
        }

        for ref fun in module.get_functions() {
            // Inspect the bytecode of every function, and if an instruction is CallGeneric,
            // verify that it calls a function with the private_generics attribute as detected earlier.
            // Then, ensure that the generic parameters of the CallGeneric instruction are valid.
            for (offset, instr) in fun.get_bytecode().iter().enumerate() {
                if let Bytecode::CallGeneric(finst_idx) = instr {
                    let FunctionInstantiation {
                        handle,
                        type_parameters,
                    } = view.function_instantiation_at(*finst_idx);

                    let fhandle = view.function_handle_at(*handle);
                    let module_handle = view.module_handle_at(fhandle.module);

                    let module_address = view
                        .address_identifier_at(module_handle.address)
                        .to_hex_literal();
                    let module_name = view.identifier_at(module_handle.name);
                    let func_name = view.identifier_at(fhandle.name).to_string();

                    let full_path_func_name =
                        format!("{}::{}::{}", module_address, module_name, func_name);

                    let type_arguments = &view.signature_at(*type_parameters).0;
                    let private_generics_types = {
                        unsafe {
                            GLOBAL_PRIVATE_GENERICS
                                .get(full_path_func_name.as_str())
                                .map(|list| list.clone())
                        }
                    };

                    if let Some(private_generics_types_indices) = private_generics_types {
                        for generic_type_index in private_generics_types_indices {
                            let type_arg = type_arguments.get(generic_type_index).unwrap();
                            let (defined_in_current_module, struct_name) =
                                is_defined_or_allowed_in_current_module(&view, type_arg);

                            let byte_loc = fun.get_bytecode_loc(offset as u16);

                            if !defined_in_current_module {
                                self.env.error(
                                    &byte_loc,
                                    format!(
                                        "resource type {:?} in function {:?} not defined in current module or not allowed",
                                        struct_name, full_path_func_name
                                    )
                                        .as_str(),
                                );
                            }
                        }
                    }
                }
            }
        }

        for (private_generics_func_name, types_list) in type_name_indices {
            let type_params_idicies = self
                .output
                .entry(compiled_module.self_id())
                .or_default()
                .private_generics_indices
                .entry(private_generics_func_name)
                .or_default();

            let _ = types_list
                .iter()
                .map(|index| type_params_idicies.push(*index))
                .collect::<Vec<_>>();
        }
    }
}

// ----------------------------------------------------------------------------------
// Entry Function

impl<'a> ExtendedChecker<'a> {
    fn check_init_module(&mut self, module: &ModuleEnv) {
        for ref fun in module.get_functions() {
            if fun.get_identifier().as_ident_str() != INIT_FN_NAME_IDENTIFIER.as_ident_str() {
                continue;
            }

            if Visibility::Private != fun.visibility() {
                self.env
                    .error(&fun.get_loc(), "module init function should private")
            }

            if fun.is_entry() {
                self.env
                    .error(&fun.get_loc(), "module init function should not entry")
            }

            if fun.get_return_count() != 0 {
                self.env.error(
                    &fun.get_loc(),
                    "module init function should not have return",
                )
            }

            let arg_tys = &fun.get_parameter_types();
            if arg_tys.len() != 1 && arg_tys.len() != 2 {
                self.env.error(
                    &fun.get_loc(),
                    "module init function should have 1 or 2 parameters",
                )
            }
            for ty in arg_tys {
                match ty {
                    Type::Reference(true, bt) => {
                        let struct_tag = bt.clone().into_struct_tag(self.env);
                        if struct_tag.is_none() {
                            self.env.error(
                                &fun.get_loc(),
                                "module init function should input a reference structure"
                            )
                        }

                        if !check_storage_context_struct_tag(struct_tag.unwrap().to_canonical_string()){
                            self.env.error(
                                &fun.get_loc(),
                                "module init function should not input reference structures other than Context"
                            )
                        }
                    }
                    Type::Reference(false, bt) => {
                        if bt.as_ref() == &Type::Primitive(PrimitiveType::Signer) {
                        } else {
                            self.env.error(
                                &fun.get_loc(),
                                "module init function should not have a reference primitive type other than a signer",
                            )
                        }
                    }
                    Type::Primitive(primitive) => {
                        if let PrimitiveType::Signer = primitive {
                        } else {
                            self.env.error(
                            &fun.get_loc(),
                            "module init function should not have a primitive type other than a signer",
                        )
                        }
                    }

                    _ => self.env.error(
                        &fun.get_loc(),
                        "module init function only should have two parameter types with signer or storageContext",
                    ),
                }
            }
        }
    }

    fn check_entry_functions(&mut self, module: &ModuleEnv) {
        for ref fun in module.get_functions() {
            if !fun.is_entry() {
                continue;
            }

            let arg_tys = &fun.get_parameter_types();
            for ty in arg_tys {
                self.check_transaction_input_type(&fun.get_loc(), ty);
            }

            if fun.get_return_count() > 0 {
                self.env
                    .error(&fun.get_loc(), "entry function cannot return values")
            }
        }
    }

    fn check_transaction_input_type(&self, loc: &Loc, ty: &Type) {
        use Type::*;
        match ty {
            Primitive(_) | TypeParameter(_) => {
                // Any primitive type allowed, any parameter expected to instantiate with primitive
            }
            Vector(ety) => {
                // Vectors are allowed if element type is allowed
                self.check_transaction_input_type(loc, ety)
            }

            Struct(mid, sid, _)
                if is_allowed_input_struct(
                    self.env
                        .get_struct(mid.qualified(*sid))
                        .get_full_name_with_address(),
                    false,
                ) =>
            {
                // Specific struct types are allowed
            }
            Reference(false, bt)
                if matches!(bt.as_ref(), Primitive(PrimitiveType::Signer))
                    || self.is_allowed_reference_types(bt) =>
            {
                // Immutable Reference to signer and specific types is allowed
            }
            Reference(true, bt) if self.is_allowed_reference_types(bt) => {
                // Mutable references to specific types is allowed
            }
            _ => {
                // Everything else is disallowed.
                self.env.error(
                    loc,
                    &format!(
                        "type `{}` is not supported as a parameter type",
                        ty.display(&self.env.get_type_display_ctx())
                    ),
                );
            }
        }
    }

    fn is_allowed_reference_types(&self, bt: &Type) -> bool {
        match bt {
            Type::Struct(mid, sid, _) => {
                let struct_name = self
                    .env
                    .get_struct(mid.qualified(*sid))
                    .get_full_name_with_address();
                if is_allowed_input_struct(struct_name, true) {
                    return true;
                }
                false
            }
            _ => false,
        }
    }
}

pub fn is_allowed_input_struct(name: String, is_ref: bool) -> bool {
    matches!(
        name.as_str(),
        "0x1::string::String"
            | "0x1::ascii::String"
            | "0x2::object::ObjectID"
            | "0x2::context::Context"
    ) ||
    // Object<T> only support passing argument by-ref, not by-value
     (is_ref && name.as_str() == "0x2::object::Object")
}

// ----------------------------------------------------------------------------------
// Check Global Storage Access

impl<'a> ExtendedChecker<'a> {
    fn check_global_storage_access(&mut self, module: &ModuleEnv) {
        for ref fun in module.get_functions() {
            let mut invalid_bytecode = vec![];
            for instr in fun.get_bytecode().iter() {
                match instr {
                    Bytecode::MoveFrom(_)
                    | Bytecode::MoveFromGeneric(_)
                    | Bytecode::MoveTo(_)
                    | Bytecode::MoveToGeneric(_)
                    | Bytecode::ImmBorrowGlobal(_)
                    | Bytecode::MutBorrowGlobal(_)
                    | Bytecode::ImmBorrowGlobalGeneric(_)
                    | Bytecode::MutBorrowGlobalGeneric(_)
                    | Bytecode::Exists(_)
                    | Bytecode::ExistsGeneric(_) => {
                        invalid_bytecode.push(instr);
                    }
                    Bytecode::Pop
                    | Bytecode::Ret
                    | Bytecode::BrTrue(_)
                    | Bytecode::BrFalse(_)
                    | Bytecode::Branch(_)
                    | Bytecode::LdU8(_)
                    | Bytecode::LdU16(_)
                    | Bytecode::LdU32(_)
                    | Bytecode::LdU64(_)
                    | Bytecode::LdU128(_)
                    | Bytecode::LdU256(_)
                    | Bytecode::CastU8
                    | Bytecode::CastU16
                    | Bytecode::CastU32
                    | Bytecode::CastU64
                    | Bytecode::CastU128
                    | Bytecode::CastU256
                    | Bytecode::LdConst(_)
                    | Bytecode::LdTrue
                    | Bytecode::LdFalse
                    | Bytecode::CopyLoc(_)
                    | Bytecode::MoveLoc(_)
                    | Bytecode::StLoc(_)
                    | Bytecode::Call(_)
                    | Bytecode::CallGeneric(_)
                    | Bytecode::Pack(_)
                    | Bytecode::PackGeneric(_)
                    | Bytecode::Unpack(_)
                    | Bytecode::UnpackGeneric(_)
                    | Bytecode::ReadRef
                    | Bytecode::WriteRef
                    | Bytecode::FreezeRef
                    | Bytecode::MutBorrowLoc(_)
                    | Bytecode::ImmBorrowLoc(_)
                    | Bytecode::MutBorrowField(_)
                    | Bytecode::MutBorrowFieldGeneric(_)
                    | Bytecode::ImmBorrowField(_)
                    | Bytecode::ImmBorrowFieldGeneric(_)
                    | Bytecode::Add
                    | Bytecode::Sub
                    | Bytecode::Mul
                    | Bytecode::Mod
                    | Bytecode::Div
                    | Bytecode::BitOr
                    | Bytecode::BitAnd
                    | Bytecode::Xor
                    | Bytecode::Shl
                    | Bytecode::Shr
                    | Bytecode::Or
                    | Bytecode::And
                    | Bytecode::Not
                    | Bytecode::Eq
                    | Bytecode::Neq
                    | Bytecode::Lt
                    | Bytecode::Gt
                    | Bytecode::Le
                    | Bytecode::Ge
                    | Bytecode::Abort
                    | Bytecode::Nop
                    | Bytecode::VecPack(_, _)
                    | Bytecode::VecLen(_)
                    | Bytecode::VecImmBorrow(_)
                    | Bytecode::VecMutBorrow(_)
                    | Bytecode::VecPushBack(_)
                    | Bytecode::VecPopBack(_)
                    | Bytecode::VecUnpack(_, _)
                    | Bytecode::VecSwap(_) => {}
                }
            }

            if !invalid_bytecode.is_empty() {
                let func_name = self
                    .env
                    .symbol_pool()
                    .string(fun.get_name())
                    .as_str()
                    .to_string();

                let error_msg = format!(
                    "Access to Move global storage is not allowed. Found in function {}: {:?}",
                    func_name, invalid_bytecode,
                );

                self.env.error(&fun.get_loc(), error_msg.as_str());
            }
        }
    }
}

impl<'a> ExtendedChecker<'a> {
    fn check_gas_free_function(&mut self, module: &ModuleEnv) {
        for fenv in module.get_functions() {
            if self.has_attribute(&fenv, GAS_FREE_ATTRIBUTE) {
                let attributes = fenv.get_attributes();
                let mut attribute_gas_validate_found = false;
                let mut attribute_gas_charge_post_found = false;

                let mut gas_free_function_map: BTreeMap<String, GasFreeFunction> = BTreeMap::new();

                // verify and save functions with the attribute #[gas_free]
                for attr in attributes {
                    if let Attribute::Apply(_, _, attribute_list) = attr {
                        for assign_attribute in attribute_list {
                            if let Attribute::Assign(_, symbol, attribute_value) = assign_attribute
                            {
                                let attribute_key =
                                    module.symbol_pool().string(*symbol).to_string();

                                if attribute_key == GAS_FREE_VALIDATE {
                                    if attribute_gas_validate_found {
                                        self.env.error(
                                            &fenv.get_loc(),
                                            "duplicate attribute key 'gas_validate'",
                                        )
                                    } else {
                                        attribute_gas_validate_found = true;
                                    }
                                }

                                if attribute_key == GAS_FREE_CHARGE_POST {
                                    if attribute_gas_charge_post_found {
                                        self.env.error(
                                            &fenv.get_loc(),
                                            "duplicate attribute key 'gas_charge_post'",
                                        )
                                    } else {
                                        attribute_gas_charge_post_found = true;
                                    }
                                }

                                if let AttributeValue::Name(
                                    _,
                                    module_name_opt,
                                    function_name_symbol,
                                ) = attribute_value
                                {
                                    // if there is no module name specified by user
                                    // compiler will use current module as the module name.
                                    match module_name_opt {
                                        None => {
                                            let gas_function_name = module
                                                .symbol_pool()
                                                .string(*function_name_symbol)
                                                .to_string();

                                            if let Some(gas_function) =
                                                get_module_env_function(module, &gas_function_name)
                                            {
                                                let current_module =
                                                    module.self_address().to_hex_literal();
                                                let current_module_name = fenv
                                                    .symbol_pool()
                                                    .string(fenv.module_env.get_name().name())
                                                    .to_string();
                                                let full_function_name = format!(
                                                    "{}::{}::{}",
                                                    current_module,
                                                    current_module_name,
                                                    gas_function_name
                                                );

                                                let current_module =
                                                    fenv.module_env.self_address().to_hex_literal();
                                                let current_module_name = fenv
                                                    .symbol_pool()
                                                    .string(fenv.module_env.get_name().name())
                                                    .to_string();
                                                let current_function_name = format!(
                                                    "{}::{}::{}",
                                                    current_module,
                                                    current_module_name,
                                                    module.symbol_pool().string(fenv.get_name())
                                                );

                                                let gas_free_function_info = gas_free_function_map
                                                    .entry(current_function_name)
                                                    .or_default();

                                                if attribute_key == GAS_FREE_VALIDATE {
                                                    let (is_ok, error_msg) =
                                                        check_gas_validate_function(
                                                            &gas_function,
                                                            self.env,
                                                        );
                                                    if !is_ok {
                                                        self.env.error(
                                                            &fenv.get_loc(),
                                                            error_msg.as_str(),
                                                        );
                                                    }
                                                    gas_free_function_info.gas_validate =
                                                        full_function_name.clone();
                                                }

                                                if attribute_key == GAS_FREE_CHARGE_POST {
                                                    let (is_ok, error_msg) =
                                                        check_gas_charge_post_function(
                                                            &gas_function,
                                                            self.env,
                                                        );
                                                    if !is_ok {
                                                        self.env.error(
                                                            &fenv.get_loc(),
                                                            error_msg.as_str(),
                                                        );
                                                    }
                                                    gas_free_function_info.gas_charge_post =
                                                        full_function_name;
                                                }
                                            } else {
                                                self.env.error(&fenv.get_loc(), format!("Gas function {:?} is not found in current module.", gas_function_name).as_str());
                                            }
                                        }

                                        Some(module_name_ref) => {
                                            let gas_function_name = module
                                                .symbol_pool()
                                                .string(*function_name_symbol)
                                                .to_string();
                                            if module_name_ref != module.get_name() {
                                                self.env.error(&fenv.get_loc(), format!("Gas function {:?} is not found in current module.", gas_function_name).as_str());
                                            }

                                            let current_module =
                                                fenv.module_env.self_address().to_hex_literal();
                                            let current_module_name = fenv
                                                .symbol_pool()
                                                .string(fenv.module_env.get_name().name())
                                                .to_string();
                                            let function_name = module
                                                .symbol_pool()
                                                .string(*function_name_symbol)
                                                .to_string();
                                            let full_function_name = format!(
                                                "{}::{}::{}",
                                                current_module, current_module_name, function_name
                                            );

                                            let current_module =
                                                fenv.module_env.get_full_name_str();
                                            let current_function_name = format!(
                                                "{}::{}",
                                                current_module,
                                                module.symbol_pool().string(fenv.get_name())
                                            );

                                            let gas_free_function_info = gas_free_function_map
                                                .entry(current_function_name)
                                                .or_default();

                                            if attribute_key == GAS_FREE_VALIDATE {
                                                gas_free_function_info.gas_validate =
                                                    full_function_name.clone();
                                            }
                                            if attribute_key == GAS_FREE_CHARGE_POST {
                                                gas_free_function_info.gas_charge_post =
                                                    full_function_name.clone();
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }

                if !attribute_gas_validate_found || !attribute_gas_charge_post_found {
                    self.env.error(&fenv.get_loc(),
                                   format!("The gas_validate function or gas_charge_post function for the {} function was not found.",
                                   module.symbol_pool().string(fenv.get_name())).as_str());
                }

                let module_metadata = self
                    .output
                    .entry(module.get_verified_module().self_id())
                    .or_default();
                module_metadata.gas_free_function_map = gas_free_function_map;
            }
        }
    }
}

impl<'a> ExtendedChecker<'a> {
    fn check_data_struct(&mut self, module_env: &ModuleEnv) {
        for struct_def in module_env.get_structs() {
            let struct_attributes = struct_def.get_attributes().to_vec();
            for attribute in struct_attributes.iter() {
                if let Attribute::Apply(_, symbol, _) = attribute {
                    let attr_name = module_env.symbol_pool().string(*symbol).to_string();
                    if attr_name == DATA_STRUCT_ATTRIBUTE {
                        let (error_message, is_allowed) =
                            check_data_struct_fields(&struct_def, module_env);
                        if !is_allowed {
                            self.env
                                .error(&struct_def.get_loc(), error_message.as_str());
                        }
                    }
                }
            }
        }

        let module_metadata = self
            .output
            .entry(module_env.get_verified_module().self_id())
            .or_default();

        let data_struct_map = unsafe {
            let result: BTreeMap<String, bool> = GLOBAL_DATA_STRUCT
                .iter()
                .map(|(key, value)| (key.clone(), *value))
                .collect();
            result
        };

        module_metadata.data_struct_map = data_struct_map;

        check_data_struct_func(self, module_env);
    }
}

fn check_data_struct_fields(struct_def: &StructEnv, module_env: &ModuleEnv) -> (String, bool) {
    let struct_fields = struct_def.get_fields().collect_vec();
    for field in struct_fields {
        let field_type = field.get_type();
        let field_name = module_env
            .symbol_pool()
            .string(field.get_name())
            .to_string();
        let is_allowed = check_data_struct_fields_type(&field_type, module_env);
        if !is_allowed {
            let struct_name = module_env
                .symbol_pool()
                .string(struct_def.get_name())
                .to_string();
            let full_struct_name = format!("{}::{}", module_env.get_full_name_str(), struct_name);

            return (
                format!(
                    "The field [{}] in struct {} is not allowed.",
                    field_name, full_struct_name
                ),
                is_allowed,
            );
        }
    }

    let struct_name = module_env
        .symbol_pool()
        .string(struct_def.get_name())
        .to_string();
    let full_struct_name = format!("{}::{}", module_env.get_full_name_str(), struct_name);
    unsafe {
        GLOBAL_DATA_STRUCT.insert(full_struct_name, true);
    }

    ("".to_string(), true)
}

fn check_data_struct_fields_type(field_type: &Type, module_env: &ModuleEnv) -> bool {
    return match field_type {
        Type::Primitive(_) => true,
        Type::Vector(item_type) => check_data_struct_fields_type(item_type.as_ref(), module_env),
        Type::Struct(module_id, struct_id, _) => {
            let module = module_env.env.get_module(*module_id);

            let struct_name = module_env
                .symbol_pool()
                .string(struct_id.symbol())
                .to_string();
            let full_struct_name = format!("{}::{}", module.get_full_name_str(), struct_name);

            if is_allowed_data_struct_type(&full_struct_name) {
                return true;
            }

            let struct_module = module_env.env.get_module(*module_id);
            let struct_env = struct_module.get_struct(*struct_id);
            check_data_struct_fields(&struct_env, &struct_module);

            let is_allowed_opt = unsafe { GLOBAL_DATA_STRUCT.get(full_struct_name.as_str()) };
            return if let Some(is_allowed) = is_allowed_opt {
                *is_allowed
            } else {
                false
            };
        }
        _ => false,
    };
}

fn is_allowed_data_struct_type(full_struct_name: &str) -> bool {
    matches!(
        full_struct_name,
        "0x1::string::String" | "0x1::ascii::String" | "0x2::object::ObjectID"
    )
}

fn check_data_struct_func(extended_checker: &mut ExtendedChecker, module_env: &ModuleEnv) {
    let mut type_name_indices: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut func_loc_map = BTreeMap::new();

    let compiled_module = module_env.get_verified_module();
    let view = BinaryIndexedView::Module(compiled_module);

    for ref fun in module_env.get_functions() {
        if extended_checker.has_attribute(fun, DATA_STRUCT_ATTRIBUTE) {
            let mut func_type_params_name_list = vec![];
            let type_params = fun.get_named_type_parameters();

            for t in type_params {
                let type_name = extended_checker
                    .env
                    .symbol_pool()
                    .string(t.0)
                    .as_str()
                    .to_string();
                func_type_params_name_list.push(type_name);
            }

            if func_type_params_name_list.is_empty() {
                extended_checker
                    .env
                    .error(&fun.get_loc(), "Function do not has type parameter.");
            }

            let attributes = fun.get_attributes();

            for attr in attributes {
                if let Attribute::Apply(_, _, types) = attr {
                    if types.is_empty() {
                        extended_checker.env.error(
                            &fun.get_loc(),
                            "A type name is needed for data_struct function.",
                        );
                    }

                    let mut attribute_type_index = vec![];
                    let mut attribute_type_names = vec![];

                    for (idx, type_name) in func_type_params_name_list.iter().enumerate() {
                        let _ = types
                            .iter()
                            .map(|attr| {
                                if let Attribute::Apply(_, name, _) = attr {
                                    let attribute_type_name = extended_checker
                                        .env
                                        .symbol_pool()
                                        .string(*name)
                                        .as_str()
                                        .to_string();

                                    if attribute_type_name == type_name.as_str() {
                                        attribute_type_index.push(idx);
                                        attribute_type_names.push(attribute_type_name);
                                    }
                                }
                            })
                            .collect::<Vec<_>>();
                    }

                    let _ = types
                        .iter()
                        .map(|attr| {
                            if let Attribute::Apply(_, name, _) = attr {
                                let attribute_type_name = extended_checker
                                    .env
                                    .symbol_pool()
                                    .string(*name)
                                    .as_str()
                                    .to_string();
                                if !attribute_type_names.contains(&attribute_type_name) {
                                    let func_name = extended_checker
                                        .env
                                        .symbol_pool()
                                        .string(fun.get_name())
                                        .as_str()
                                        .to_string();

                                    extended_checker.env.error(
                                        &fun.get_loc(),
                                        format!(
                                            "type name {:?} not defined in function {:?}",
                                            attribute_type_name, func_name
                                        )
                                        .as_str(),
                                    );
                                }
                            }
                        })
                        .collect::<Vec<_>>();

                    let module_address = module_env.self_address().to_hex_literal();
                    let module_name = extended_checker
                        .env
                        .symbol_pool()
                        .string(module_env.get_name().name())
                        .as_str()
                        .to_string();
                    let func_name = extended_checker
                        .env
                        .symbol_pool()
                        .string(fun.get_name())
                        .as_str()
                        .to_string();
                    let full_path_func_name =
                        format!("{}::{}::{}", module_address, module_name, func_name);
                    type_name_indices
                        .insert(full_path_func_name.clone(), attribute_type_index.clone());

                    unsafe {
                        GLOBAL_DATA_STRUCT_FUNC
                            .insert(full_path_func_name, attribute_type_index.clone());
                    }

                    func_loc_map.insert(func_name, fun.get_loc());
                }
            }
        }
    }

    let module_metadata = extended_checker
        .output
        .entry(module_env.get_verified_module().self_id())
        .or_default();

    let data_struct_func_map = unsafe {
        let result: BTreeMap<String, Vec<usize>> = GLOBAL_DATA_STRUCT_FUNC
            .iter()
            .map(|(key, value)| (key.clone(), value.clone()))
            .collect();
        result
    };

    module_metadata.data_struct_func_map = data_struct_func_map;

    for ref fun in module_env.get_functions() {
        for (_offset, instr) in fun.get_bytecode().iter().enumerate() {
            if let Bytecode::CallGeneric(finst_idx) = instr {
                let FunctionInstantiation {
                    handle,
                    type_parameters,
                } = view.function_instantiation_at(*finst_idx);

                let fhandle = view.function_handle_at(*handle);
                let module_handle = view.module_handle_at(fhandle.module);

                let module_address = view
                    .address_identifier_at(module_handle.address)
                    .to_hex_literal();

                let module_name = view.identifier_at(module_handle.name);
                let func_name = view.identifier_at(fhandle.name).to_string();

                let full_path_func_name =
                    format!("{}::{}::{}", module_address, module_name, func_name);

                let type_arguments = &view.signature_at(*type_parameters).0;

                let data_struct_func_types = {
                    unsafe {
                        GLOBAL_DATA_STRUCT_FUNC
                            .get(full_path_func_name.as_str())
                            .map(|list| list.clone())
                    }
                };

                if let Some(data_struct_func_indicies) = data_struct_func_types {
                    for generic_type_index in data_struct_func_indicies {
                        let type_arg = type_arguments.get(generic_type_index).unwrap();

                        let (is_allowed_struct_type, error_message) =
                            check_func_data_struct(&view, fun, type_arg);

                        if !is_allowed_struct_type {
                            extended_checker
                                .env
                                .error(&fun.get_loc(), error_message.as_str());
                            return;
                        }
                    }
                }
            }
        }
    }
}

fn check_func_data_struct(
    view: &BinaryIndexedView,
    func_env: &FunctionEnv,
    type_arg: &SignatureToken,
) -> (bool, String) {
    let func_name = func_env.get_full_name_str();
    match type_arg {
        SignatureToken::Struct(struct_handle_index) => {
            let shandle = view.struct_handle_at(*struct_handle_index);
            let module_id = shandle.module;
            let module_handle = view.module_handle_at(module_id);
            let module_address = view.address_identifier_at(module_handle.address);
            let module_name = view.identifier_at(module_handle.name);
            let full_module_name = format!("{}::{}", module_address.to_hex_literal(), module_name);
            let struct_name = view.identifier_at(shandle.name).to_string();
            let full_struct_name = format!(
                "{}::{}::{}",
                module_address.to_hex_literal(),
                module_name,
                struct_name
            );

            unsafe {
                let data_struct_opt = GLOBAL_DATA_STRUCT.get(full_struct_name.as_str());
                if let Some(is_data_struct) = data_struct_opt {
                    if *is_data_struct {
                        return (true, "".to_string());
                    }
                }
            }

            (false, format!("The type argument {} of #[data_struct] for function {} in the module {} is not allowed.",
            full_struct_name, func_name, full_module_name))
        }
        _ => {
            let module_id = view.self_id().unwrap().to_string();
            (false, format!("The type argument of #[data_struct] for function {} in the module {} is not allowed.",
            func_name, module_id))
        }
    }
}

fn get_module_env_function<'a>(
    module_env: &'a ModuleEnv<'a>,
    fname: &String,
) -> Option<FunctionEnv<'a>> {
    for fenv in module_env.get_functions() {
        let function_name = module_env.symbol_pool().string(fenv.get_name()).to_string();
        if &function_name == fname {
            return Some(fenv);
        }
    }
    None
}

fn check_gas_validate_function(fenv: &FunctionEnv, global_env: &GlobalEnv) -> (bool, String) {
    let params_types = fenv.get_parameter_types();
    let return_types = fenv.get_return_types();
    if params_types.is_empty() {
        return (false, "parameter length is less than 1".to_string());
    }
    if return_types.is_empty() {
        return (false, "return value length is less than 1".to_string());
    }

    let storage_ctx_type = params_types.get(0).unwrap();
    let parameter_checking_result = match storage_ctx_type {
        Type::Struct(module_id, struct_id, _) => {
            let struct_name = global_env
                .get_struct(module_id.qualified(*struct_id))
                .get_full_name_str();
            if struct_name != "0x2::context::Context" {
                (
                    false,
                    format!(
                        "Type {} cannot be used as the first parameter.",
                        struct_name
                    ),
                )
            } else {
                (true, "".to_string())
            }
        }
        _ => (
            false,
            "Only type 0x2::storage_context::StorageContext can be used as the first parameter."
                .to_string(),
        ),
    };

    if !parameter_checking_result.0 {
        return parameter_checking_result;
    }

    let first_return_type = return_types.get(0).unwrap();
    match first_return_type {
        Type::Primitive(PrimitiveType::Bool) => (true, "".to_string()),
        _ => (false, "Return type must be of type Bool.".to_string()),
    }
}

fn check_gas_charge_post_function(fenv: &FunctionEnv, global_env: &GlobalEnv) -> (bool, String) {
    let params_types = fenv.get_parameter_types();
    let return_types = fenv.get_return_types();
    if params_types.len() < 2 {
        return (false, "Length of parameters is less than 2.".to_string());
    }
    if return_types.is_empty() {
        return (false, "Length of return values is less than 1.".to_string());
    }

    let storage_ctx_type = params_types.get(0).unwrap();
    match storage_ctx_type {
        Type::Struct(module_id, struct_id, _) => {
            let struct_name = global_env
                .get_struct(module_id.qualified(*struct_id))
                .get_full_name_str();
            if struct_name != "0x2::storage_context::StorageContext" {
                return (
                    false,
                    format!(
                        "Type {} cannot be used as the first parameter.",
                        struct_name
                    ),
                );
            }
        }
        _ => return (
            false,
            "Only type 0x2::storage_context::StorageContext can be used as the first parameter."
                .to_string(),
        ),
    }

    let gas_used_type = params_types.get(1).unwrap();
    let second_parameter_checking_result = match *gas_used_type {
        Type::Primitive(PrimitiveType::U256) => (true, "".to_string()),
        _ => (
            false,
            "The second parameter must be of type U256.".to_string(),
        ),
    };

    if !second_parameter_checking_result.0 {
        return second_parameter_checking_result;
    }

    let first_return_type = return_types.get(0).unwrap();
    match first_return_type {
        Type::Primitive(PrimitiveType::Bool) => (true, "".to_string()),
        _ => (false, "Return type must be of type Bool.".to_string()),
    }
}

// ----------------------------------------------------------------------------------
// Helpers

impl<'a> ExtendedChecker<'a> {
    fn has_attribute(&self, fun: &FunctionEnv, attr_name: &str) -> bool {
        fun.get_attributes().iter().any(|attr| {
            if let Attribute::Apply(_, name, _) = attr {
                self.env.symbol_pool().string(*name).as_str() == attr_name
            } else {
                false
            }
        })
    }
}

pub fn check_storage_context_struct_tag(struct_full_name: String) -> bool {
    struct_full_name == Context::struct_tag().to_canonical_string()
}

pub fn is_defined_or_allowed_in_current_module(
    view: &BinaryIndexedView,
    type_arg: &SignatureToken,
) -> (bool, String) {
    match type_arg {
        SignatureToken::Struct(idx) | SignatureToken::StructInstantiation(idx, _) => {
            let shandle = view.struct_handle_at(*idx);
            let struct_name = view.identifier_at(shandle.name).to_string();

            if view.self_handle_idx() == Some(shandle.module) {
                return (true, struct_name);
            }

            (false, struct_name)
        }
        // Other types are not allowed.
        SignatureToken::TypeParameter(tidx) => (false, format!("T{}", *tidx as usize)),
        SignatureToken::Bool => (false, "Bool".to_owned()),
        SignatureToken::U8 => (false, "U8".to_owned()),
        SignatureToken::U16 => (false, "U16".to_owned()),
        SignatureToken::U32 => (false, "U32".to_owned()),
        SignatureToken::U64 => (false, "U64".to_owned()),
        SignatureToken::U128 => (false, "U128".to_owned()),
        SignatureToken::U256 => (false, "U256".to_owned()),
        SignatureToken::Signer => (false, "Signer".to_owned()),
        SignatureToken::Address => (false, "Address".to_owned()),
        SignatureToken::Vector(_) => (false, "Vector".to_owned()),
        SignatureToken::Reference(_) => (false, "Reference".to_owned()),
        SignatureToken::MutableReference(_) => (false, "MutableReference".to_owned()),
    }
}

#[derive(Clone, Debug, PartialEq, Eq, Error)]
pub enum MalformedError {
    #[error("Unknown key found: {0:?}")]
    UnknownKey(Vec<u8>),
    #[error("Unable to deserialize value for {0:?}: {1}")]
    DeserializedError(Vec<u8>, bcs::Error),
    #[error("Duplicate key for metadata")]
    DuplicateKey,
}

/// Check if the metadata has unknown key/data types
pub fn check_metadata_format(module: &CompiledModule) -> Result<(), MalformedError> {
    let mut exist = false;
    for data in module.metadata.iter() {
        if data.key == ROOCH_METADATA_KEY {
            if exist {
                return Err(MalformedError::DuplicateKey);
            }
            exist = true;

            if data.key == *ROOCH_METADATA_KEY {
                bcs::from_bytes::<RuntimeModuleMetadataV1>(&data.value)
                    .map_err(|e| MalformedError::DeserializedError(data.key.clone(), e))?;
            }
        } else {
            return Err(MalformedError::UnknownKey(data.key.clone()));
        }
    }

    Ok(())
}

pub fn load_module_metadata(
    module_id: &ModuleId,
    loaded_module_bytes: VMResult<Vec<u8>>,
) -> VMResult<Option<RuntimeModuleMetadataV1>> {
    let compiled_module_opt = {
        match loaded_module_bytes {
            Ok(module_bytes) => CompiledModule::deserialize(module_bytes.as_slice()).ok(),
            Err(err) => {
                return Err(err);
            }
        }
    };

    match compiled_module_opt {
        None => Err(
            PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_RESOURCE)
                .with_message(format!(
                    "failed to deserialize module {:?}",
                    module_id.to_string()
                ))
                .finish(Location::Module(module_id.clone())),
        ),
        Some(compiled_module) => Ok(get_metadata_from_compiled_module(&compiled_module)),
    }
}
