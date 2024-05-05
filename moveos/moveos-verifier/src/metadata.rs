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
    Ability, Bytecode, FunctionHandleIndex, FunctionInstantiation, SignatureToken, Visibility,
};
use move_binary_format::CompiledModule;
use move_core_types::language_storage::ModuleId;
use move_core_types::metadata::Metadata;
use move_core_types::vm_status::StatusCode;
use move_model::ast::{Attribute, AttributeValue};
use move_model::model::{FunctionEnv, GlobalEnv, Loc, ModuleEnv, StructEnv};
use move_model::ty::Type;
use move_model::ty::{PrimitiveType, ReferenceKind};
use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::ops::Deref;
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
            && self.data_struct_map.is_empty()
            && self.data_struct_func_map.is_empty()
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
        Some(RuntimeModuleMetadataV1::default())
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
        let mut func_loc_map = BTreeMap::new();

        let compiled_module = match module.get_verified_module() {
            None => {
                return;
            }
            Some(module) => module,
        };

        let view = BinaryIndexedView::Module(compiled_module);

        // The `type_name_indices` is used to save the private_generics information of the found function to the metadata of the module.
        // The private_generics information of the function is looked up from `GLOBAL_PRIVATE_GENERICS`.
        let type_name_indices = get_type_name_indices(self.env, module, &mut func_loc_map);

        let type_name_lists = verify_the_type_name_lists(self.env, module);

        if type_name_indices.len() != type_name_lists.len() {
            self.env.error(
                &module.get_loc(),
                "some private_generics may not have been handled.",
            );
        }

        for (full_func_name, types_size) in type_name_lists.iter() {
            match type_name_indices.get(full_func_name) {
                None => {
                    self.env.error(
                        &module.get_loc(),
                        format!("the function {:?} may not exists.", full_func_name).as_str(),
                    );
                }
                Some(type_list) => {
                    if type_list.len() as u32 != *types_size {
                        self.env.error(
                            &module.get_loc(),
                            "some type name in private_generics may not have been handled",
                        );
                    }
                }
            }
        }

        // Inspect the bytecode of every function, and if an instruction is CallGeneric,
        // verify that it calls a function with the private_generics attribute as detected earlier.
        // Then, ensure that the generic parameters of the CallGeneric instruction are valid.
        check_call_generics(self.env, module, view);

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

fn verify_the_type_name_lists(
    global_env: &GlobalEnv,
    module_env: &ModuleEnv,
) -> BTreeMap<String, u32> {
    let mut total_type_name_map: BTreeMap<String, u32> = BTreeMap::new();

    for ref fun in module_env.get_functions() {
        let full_func_name = build_full_func_name(fun, module_env, global_env);
        if has_attribute(global_env, fun, PRIVATE_GENERICS_ATTRIBUTE) {
            let attributes = fun.get_attributes();

            for attr in attributes {
                if let Attribute::Apply(_, _, types) = attr {
                    if !types.is_empty() {
                        total_type_name_map.insert(full_func_name.clone(), types.len() as u32);
                    }
                }
            }
        }
    }

    total_type_name_map
}

fn get_type_name_indices(
    global_env: &GlobalEnv,
    module: &ModuleEnv,
    func_loc_map: &mut BTreeMap<String, Loc>,
) -> BTreeMap<String, Vec<usize>> {
    let mut type_name_indices = BTreeMap::new();

    // Check every function and if a function has the private_generics attribute,
    // ensure that the function name and the types defined in the private_generics attribute match,
    // for example: #[private_generics(T1, T2)].
    for ref fun in module.get_functions() {
        let full_func_name = build_full_func_name(fun, module, global_env);

        if has_attribute(global_env, fun, PRIVATE_GENERICS_ATTRIBUTE) {
            let mut func_type_params_name_list = vec![];

            let type_params = fun.get_type_parameters();
            for t in type_params {
                let type_name = global_env.symbol_pool().string(t.0).as_str().to_string();
                func_type_params_name_list.push(type_name);
            }

            if func_type_params_name_list.is_empty() {
                global_env.error(&fun.get_loc(), "Function do not has type parameter.");
            }

            let attributes = fun.get_attributes();

            for attr in attributes {
                if let Attribute::Apply(_, _, types) = attr {
                    if types.is_empty() {
                        global_env.error(
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
                                    let attribute_type_name =
                                        global_env.symbol_pool().string(*name).as_str().to_string();

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
                                    global_env.symbol_pool().string(*name).as_str().to_string();

                                if !attribute_type_names.contains(&attribute_type_name) {
                                    global_env.error(
                                        &fun.get_loc(),
                                        format!(
                                            "type name {:?} not defined in function {:?}",
                                            attribute_type_name, full_func_name
                                        )
                                        .as_str(),
                                    );
                                }
                            }
                        })
                        .collect::<Vec<_>>();

                    type_name_indices.insert(full_func_name.clone(), attribute_type_index.clone());

                    unsafe {
                        GLOBAL_PRIVATE_GENERICS
                            .insert(full_func_name.clone(), attribute_type_index.clone());
                    }

                    func_loc_map.insert(full_func_name.clone(), fun.get_loc());
                }
            }
        }
    }

    type_name_indices
}

fn build_full_func_name(fun: &FunctionEnv, module: &ModuleEnv, global_env: &GlobalEnv) -> String {
    let module_address = module.self_address().expect_numerical().to_hex_literal();
    let module_name = global_env
        .symbol_pool()
        .string(module.get_name().name())
        .as_str()
        .to_string();
    let func_name = global_env
        .symbol_pool()
        .string(fun.get_name())
        .as_str()
        .to_string();

    format!("{}::{}::{}", module_address, module_name, func_name)
}

fn check_call_generics(global_env: &GlobalEnv, module: &ModuleEnv, view: BinaryIndexedView) {
    for ref fun in module.get_functions() {
        if fun.is_inline() {
            // The fun.get_bytecode() will only be None when the function is an inline function.
            // So we need to skip inline functions.
            continue;
        }

        for (offset, instr) in fun.get_bytecode().unwrap().iter().enumerate() {
            if let Bytecode::CallGeneric(finst_idx) = instr {
                let FunctionInstantiation {
                    handle: fhandle_index,
                    type_parameters,
                } = view.function_instantiation_at(*finst_idx);

                let full_path_func_name = get_func_name_for_fhandle(fhandle_index, &view);

                let func_type_arguments = &view.signature_at(*type_parameters).0;
                let private_generics_types = {
                    unsafe {
                        GLOBAL_PRIVATE_GENERICS
                            .get(full_path_func_name.as_str())
                            .map(|list| list.clone())
                    }
                };

                // if the called function have the private_generics information.
                if let Some(private_generics_types_list) = private_generics_types {
                    let byte_loc = fun
                        .get_bytecode_loc(offset as u16)
                        .unwrap_or_else(|| Loc::default());

                    // We iterate over the type index from the private_generics information list
                    // and query the type index in the function's type arguments.
                    for generic_type_index in private_generics_types_list {
                        let type_arg = match func_type_arguments.get(generic_type_index) {
                            None => {
                                global_env.error(
                                    &byte_loc,
                                    format!(
                                        "the function {:?} does not have enough type parameters.",
                                        full_path_func_name
                                    )
                                    .as_str(),
                                );
                                return;
                            }
                            Some(sig_token) => sig_token,
                        };

                        // Report an error if the type is not defined in the current module or is not allowed.
                        let (defined_in_current_module, struct_name) =
                            is_defined_or_allowed_in_current_module(&view, type_arg);

                        if !defined_in_current_module {
                            global_env.error(
                                &byte_loc,
                                format!(
                                    "resource type {:?} in function {:?} not defined in current module or not allowed",
                                    struct_name, full_path_func_name
                                ).as_str(),
                            );
                        }
                    }
                }
            }
        }
    }
}

fn get_func_name_for_fhandle(
    fhandle_index: &FunctionHandleIndex,
    view: &BinaryIndexedView,
) -> String {
    let fhandle = view.function_handle_at(*fhandle_index);
    let module_handle = view.module_handle_at(fhandle.module);

    let module_address = view
        .address_identifier_at(module_handle.address)
        .to_hex_literal();
    let module_name = view.identifier_at(module_handle.name);
    let func_name = view.identifier_at(fhandle.name).to_string();

    format!("{}::{}::{}", module_address, module_name, func_name)
}

// ----------------------------------------------------------------------------------
// Entry Function

impl<'a> ExtendedChecker<'a> {
    fn check_init_module(&mut self, module: &ModuleEnv) {
        for ref fun in module.get_functions() {
            if fun.is_inline() {
                // The fun.get_identifier() will only be None when the function is an inline function.
                // So we need to skip inline functions.
                continue;
            }
            if fun.get_identifier().unwrap().as_ident_str()
                != INIT_FN_NAME_IDENTIFIER.as_ident_str()
            {
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
            if arg_tys.len() > 1 {
                self.env.error(
                    &fun.get_loc(),
                    "module init function can only have up to 1 signer parameters",
                )
            }
            for ty in arg_tys {
                match ty {
                    Type::Reference(ReferenceKind::Mutable, _bt) => {
                        self.env.error(
                            &fun.get_loc(),
                            "module init function should not have mutable reference type as parameter",
                        );
                    }
                    Type::Reference(ReferenceKind::Immutable, bt) => {
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
                        "module init function only should have one parameter types with signer",
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
            Reference(ReferenceKind::Immutable, bt)
                if matches!(bt.as_ref(), Primitive(PrimitiveType::Signer))
                    || self.is_allowed_reference_types(bt) =>
            {
                // Immutable Reference to signer and specific types is allowed
            }
            Reference(ReferenceKind::Mutable, bt) if self.is_allowed_reference_types(bt) => {
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
    ) ||
    // Object<T> only support passing argument by-ref, not by-value
     (is_ref && name.as_str() == "0x2::object::Object")
}

// ----------------------------------------------------------------------------------
// Check Global Storage Access

impl<'a> ExtendedChecker<'a> {
    fn check_global_storage_access(&mut self, module: &ModuleEnv) {
        for ref fun in module.get_functions() {
            if fun.is_inline() {
                // The fun.get_bytecode() will only be None when the function is an inline function.
                // So we need to skip inline functions.
                continue;
            }
            let mut invalid_bytecode = vec![];
            for instr in fun.get_bytecode().unwrap() {
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
            if has_attribute(self.env, &fenv, GAS_FREE_ATTRIBUTE) {
                // TODO: gas_free attribute is not supported yet.
                // Remove this when it's ready
                self.env
                    .error(&fenv.get_loc(), "Unsupported attribute 'gas_free'.");
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
                                                let current_module = module
                                                    .self_address()
                                                    .expect_numerical()
                                                    .to_hex_literal();
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

                                                let current_module = fenv
                                                    .module_env
                                                    .self_address()
                                                    .expect_numerical()
                                                    .to_hex_literal();
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

                                            let current_module = fenv
                                                .module_env
                                                .self_address()
                                                .expect_numerical()
                                                .to_hex_literal();
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

                let verified_module = match module.get_verified_module() {
                    None => {
                        self.env
                            .error(&fenv.get_loc(), "The verified module was not found.");
                        return;
                    }
                    Some(module) => module,
                };

                let module_metadata = self.output.entry(verified_module.self_id()).or_default();
                module_metadata.gas_free_function_map = gas_free_function_map;
            }
        }
    }
}

impl<'a> ExtendedChecker<'a> {
    fn check_data_struct(&mut self, module_env: &ModuleEnv) {
        for struct_def in module_env.get_structs() {
            if is_data_struct_annotation(&struct_def, module_env) {
                if is_copy_drop_struct(&struct_def) {
                    let (error_message, is_allowed) =
                        check_data_struct_fields(&struct_def, module_env);
                    if !is_allowed {
                        self.env
                            .error(&struct_def.get_loc(), error_message.as_str());
                    }
                } else {
                    let struct_name = struct_def.get_full_name_str();
                    self.env.error(
                        &struct_def.get_loc(),
                        format!(
                            "The struct {} must have the 'copy' and 'drop' ability",
                            struct_name
                        )
                        .as_str(),
                    );
                }
            }
        }

        let verified_module = match module_env.get_verified_module() {
            None => {
                self.env
                    .error(&module_env.get_loc(), "The verified module was not found.");
                return;
            }
            Some(module) => module,
        };

        let module_metadata = self.output.entry(verified_module.self_id()).or_default();

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
        Type::Struct(module_id, struct_id, ty_args) => {
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

            if is_std_option_type(&full_struct_name) {
                if let Some(item_type) = ty_args.first() {
                    return check_data_struct_fields_type(item_type, module_env);
                }

                return false;
            }

            if !is_data_struct_annotation(&struct_env, module_env) {
                return false;
            }

            if !is_copy_drop_struct(&struct_env) {
                return false;
            }

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

fn is_std_option_type(full_struct_name: &str) -> bool {
    matches!(full_struct_name, "0x1::option::Option")
}

fn is_data_struct_annotation(struct_env: &StructEnv, module_env: &ModuleEnv) -> bool {
    let struct_attributes = struct_env.get_attributes().to_vec();
    for attribute in struct_attributes.iter() {
        if let Attribute::Apply(_, symbol, _) = attribute {
            let attr_name = module_env.symbol_pool().string(*symbol).to_string();
            if attr_name == DATA_STRUCT_ATTRIBUTE {
                return true;
            }
        }
    }

    false
}

fn is_copy_drop_struct(struct_env: &StructEnv) -> bool {
    let abilities = struct_env.get_abilities();
    if abilities.has_ability(Ability::Copy) && abilities.has_ability(Ability::Drop) {
        return true;
    }

    false
}

fn check_data_struct_func(extended_checker: &mut ExtendedChecker, module_env: &ModuleEnv) {
    let mut type_name_indices: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    let mut func_loc_map = BTreeMap::new();

    let compiled_module = match module_env.get_verified_module() {
        None => {
            module_env
                .env
                .error(&module_env.get_loc(), "The verified module was not found.");
            return;
        }
        Some(module) => module,
    };

    let view = BinaryIndexedView::Module(compiled_module);

    for ref fun in module_env.get_functions() {
        if has_attribute(extended_checker.env, fun, DATA_STRUCT_ATTRIBUTE) {
            let mut func_type_params_name_list = vec![];
            let type_params = fun.get_type_parameters();

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

                    let module_address = module_env
                        .self_address()
                        .expect_numerical()
                        .to_hex_literal();
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

    let compiled_module = match module_env.get_verified_module() {
        None => {
            module_env
                .env
                .error(&module_env.get_loc(), "The verified module was not found.");
            return;
        }
        Some(module) => module,
    };

    let module_metadata = extended_checker
        .output
        .entry(compiled_module.self_id())
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
        if fun.is_inline() {
            // The fun.get_bytecode() will only be None when the function is an inline function.
            // So we need to skip inline functions.
            continue;
        }
        for instr in fun.get_bytecode().unwrap().iter() {
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
                        let type_arg = match type_arguments.get(generic_type_index) {
                            None => {
                                extended_checker.env.error(&fun.get_loc(), "");
                                return;
                            }
                            Some(v) => v,
                        };

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

            // #[data_struct(T)] supports not only structs, but also string and ObjectID.
            if is_allowed_data_struct_type(&full_struct_name) {
                return (true, "".to_string());
            }

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
        SignatureToken::StructInstantiation(struct_handle_index, ty_args) => {
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

            // #[data_struct(T)] supports not only structs, but also std::option::Option.
            if is_std_option_type(&full_struct_name) {
                if let Some(item_type) = ty_args.first() {
                    return check_func_data_struct(view, func_env, item_type)
                }
            }

            (false, format!("The type argument {} of #[data_struct] for function {} in the module {} is not allowed.",
            full_struct_name, func_name, full_module_name))
        }
        // #[data_struct(T)] supports not only structs, but also primitive types and vectors.
        SignatureToken::Vector(item_type) => {
            check_func_data_struct(view, func_env, item_type.deref())
        }
        SignatureToken::Address => (true, "".to_string()),
        SignatureToken::Bool => (true, "".to_string()),
        SignatureToken::U8 => (true, "".to_string()),
        SignatureToken::U16 => (true, "".to_string()),
        SignatureToken::U32 => (true, "".to_string()),
        SignatureToken::U64 => (true, "".to_string()),
        SignatureToken::U128 => (true, "".to_string()),
        SignatureToken::U256 => (true, "".to_string()),
        _ => {
            // Only when the view is a Script, will view.self_id() return None.
            // However, in our case, we are dealing with a CompiledModule, so it won't be None here.
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

fn check_gas_validate_function(fenv: &FunctionEnv, _global_env: &GlobalEnv) -> (bool, String) {
    let params_types = fenv.get_parameter_types();
    let return_type_count = fenv.get_return_count();
    let mut return_types = vec![];
    for i in 0..return_type_count {
        return_types.push(fenv.get_result_type_at(i));
    }

    if params_types.is_empty() {
        return (false, "parameter length is less than 1".to_string());
    }
    if return_types.is_empty() {
        return (false, "return value length is less than 1".to_string());
    }
    //TODO FIXME

    // Length of the return_types array has already been checked above, so unwrap directly here.
    let first_return_type = return_types.first().unwrap();
    match first_return_type {
        Type::Primitive(PrimitiveType::Bool) => (true, "".to_string()),
        _ => (false, "Return type must be of type Bool.".to_string()),
    }
}

fn check_gas_charge_post_function(fenv: &FunctionEnv, _global_env: &GlobalEnv) -> (bool, String) {
    let params_types = fenv.get_parameter_types();
    let return_type_count = fenv.get_return_count();
    let mut return_types = vec![];
    for i in 0..return_type_count {
        return_types.push(fenv.get_result_type_at(i));
    }

    if params_types.len() < 2 {
        return (false, "Length of parameters is less than 2.".to_string());
    }
    if return_types.is_empty() {
        return (false, "Length of return values is less than 1.".to_string());
    }

    //TODO FIXME

    // Length of the params_types array has already been checked above, so unwrap directly here.
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

    // Length of the return_types array has already been checked above, so unwrap directly here.
    let first_return_type = return_types.first().unwrap();
    match first_return_type {
        Type::Primitive(PrimitiveType::Bool) => (true, "".to_string()),
        _ => (false, "Return type must be of type Bool.".to_string()),
    }
}

// ----------------------------------------------------------------------------------
// Helpers

fn has_attribute(global_env: &GlobalEnv, fun: &FunctionEnv, attr_name: &str) -> bool {
    fun.get_attributes().iter().any(|attr| {
        if let Attribute::Apply(_, name, _) = attr {
            global_env.symbol_pool().string(*name).as_str() == attr_name
        } else {
            false
        }
    })
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
                    .map_err(|e| MalformedError::DeserializedError(data.key.clone(), e.clone()))?;
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
