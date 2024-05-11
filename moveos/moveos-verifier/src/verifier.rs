// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use std::collections::BTreeMap;
use std::ops::Deref;

use move_binary_format::binary_views::BinaryIndexedView;
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMError, VMResult};
use move_binary_format::file_format::{
    Bytecode, FunctionDefinition, FunctionDefinitionIndex, FunctionHandleIndex,
    FunctionInstantiation, FunctionInstantiationIndex, Signature, SignatureToken,
    StructHandleIndex, Visibility,
};
use move_binary_format::IndexKind;
use move_binary_format::{access::ModuleAccess, CompiledModule};
use move_core_types::identifier::Identifier;
use move_core_types::language_storage::ModuleId;
use move_core_types::resolver::ModuleResolver;
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_runtime::session::{LoadedFunctionInstantiation, Session};
use move_vm_types::loaded_data::runtime_types::Type;
use once_cell::sync::Lazy;

use crate::metadata::{
    check_metadata_format, get_metadata_from_compiled_module, is_allowed_input_struct,
    is_defined_or_allowed_in_current_module,
};

pub static INIT_FN_NAME_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("init").unwrap());

pub fn verify_modules<Resolver>(modules: &Vec<CompiledModule>, db: Resolver) -> VMResult<bool>
where
    Resolver: ModuleResolver,
{
    let mut verified_modules: BTreeMap<ModuleId, CompiledModule> = BTreeMap::new();
    for module in modules {
        verify_private_generics(module, &db, &mut verified_modules)?;
        verify_entry_function_at_publish(module)?;
        verify_global_storage_access(module)?;
        verify_gas_free_function(module)?;
        verify_data_struct(module, &db, &mut verified_modules)?;
        verify_init_function(module)?;
    }

    Ok(true)
}

pub fn verify_module<Resolver>(
    module: &CompiledModule,
    db: Resolver,
    verified_modules: &mut BTreeMap<ModuleId, CompiledModule>,
) -> VMResult<bool>
where
    Resolver: ModuleResolver,
{
    verify_private_generics(module, &db, verified_modules)?;
    verify_entry_function_at_publish(module)?;
    verify_global_storage_access(module)?;
    verify_gas_free_function(module)?;
    verify_data_struct(module, &db, verified_modules)?;
    verify_init_function(module)
}

/// The initializer function must have the following properties in order to be executed at publication:
/// - Name init
/// - Single parameter of &mut TxContext type
/// - No return values
/// - Private
pub fn verify_init_function(module: &CompiledModule) -> VMResult<bool> {
    for fdef in &module.function_defs {
        let fhandle = module.function_handle_at(fdef.function);
        let fname = module.identifier_at(fhandle.name);
        if fname == INIT_FN_NAME_IDENTIFIER.as_ident_str() {
            if Visibility::Private != fdef.visibility {
                return Err(vm_error_for_init_func_checking(
                    StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
                    "init function should private",
                    fdef,
                    module.self_id(),
                ));
            }

            if fdef.is_entry {
                return Err(vm_error_for_init_func_checking(
                    StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE,
                    "init function should not entry function",
                    fdef,
                    module.self_id(),
                ));
            }

            let view = BinaryIndexedView::Module(module);
            let func_parameter_signatures = view.signature_at(fhandle.parameters);
            let func_parameter_vec = &func_parameter_signatures.0;

            if func_parameter_vec.len() > 1 {
                return Err(vm_error_for_init_func_checking(
                    StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH,
                    "init function only should have one parameter with signer",
                    fdef,
                    module.self_id(),
                ));
            }

            for st in func_parameter_vec {
                let is_allowed = is_allowed_init_func_param(&view, st);
                if !is_allowed {
                    return Err(vm_error_for_init_func_checking(
                        StatusCode::TYPE_MISMATCH,
                        "init function should only enter signer",
                        fdef,
                        module.self_id(),
                    ));
                }
            }

            return Ok(true);
        }
    }
    Ok(false)
}

fn is_allowed_init_func_param(_module_view: &BinaryIndexedView, st: &SignatureToken) -> bool {
    if st == &SignatureToken::Signer {
        true
    } else {
        match st {
            SignatureToken::Reference(inner_st) => inner_st.as_ref() == &SignatureToken::Signer,
            _ => false,
        }
    }
}

pub fn verify_entry_function_at_publish(module: &CompiledModule) -> VMResult<bool> {
    let module_bin_view = BinaryIndexedView::Module(module);

    for fdef in module.function_defs.iter() {
        if !fdef.is_entry {
            continue;
        }

        let function_handle = module_bin_view.function_handle_at(fdef.function);
        let return_types = module_bin_view
            .signature_at(function_handle.return_)
            .0
            .clone();
        if !return_types.is_empty() {
            return Err(
                PartialVMError::new(StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE)
                    .with_message("function should not return values".to_owned())
                    .finish(Location::Module(module.self_id())),
            );
        }

        let func_parameters_types = module_bin_view
            .signature_at(function_handle.parameters)
            .0
            .clone();

        for (idx, ty) in func_parameters_types.iter().enumerate() {
            if !check_transaction_input_type_at_publish(ty, &module_bin_view) {
                return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                    .with_message(format!("The type of the {} parameter is not allowed", idx))
                    .at_index(IndexKind::FunctionDefinition, fdef.function.0)
                    .finish(Location::Module(module.self_id())));
            }
        }
    }

    Ok(true)
}

pub fn verify_entry_function<S>(
    func: &LoadedFunctionInstantiation,
    session: &Session<S>,
) -> PartialVMResult<()>
where
    S: TransactionCache,
{
    if !func.return_.is_empty() {
        return Err(
            PartialVMError::new(StatusCode::INVALID_MAIN_FUNCTION_SIGNATURE)
                .with_message("function should not return values".to_owned()),
        );
    }

    for (idx, ty) in func.parameters.iter().enumerate() {
        if !check_transaction_input_type(ty, session) {
            return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                .with_message(format!("The type of the {} parameter is not allowed", idx)));
        }
    }

    Ok(())
}

fn check_transaction_input_type_at_publish(
    ety: &SignatureToken,
    module_bin_view: &BinaryIndexedView,
) -> bool {
    use SignatureToken::*;
    match ety {
        Bool | U8 | U16 | U32 | U64 | U128 | U256 | Address | Signer => true,
        Vector(ety) => check_transaction_input_type_at_publish(ety.deref(), module_bin_view),
        Reference(bt)
            if matches!(bt.as_ref(), Signer)
                || is_allowed_reference_types_at_publish(bt.as_ref(), module_bin_view) =>
        {
            true
        }
        MutableReference(bt)
            if is_allowed_reference_types_at_publish(bt.as_ref(), module_bin_view) =>
        {
            true
        }
        Struct(sid) | StructInstantiation(sid, _) => {
            let struct_full_name = struct_full_name_from_sid(sid, module_bin_view);
            is_allowed_input_struct(struct_full_name, false)
        }

        _ => {
            // Everything else is disallowed.
            false
        }
    }
}

fn is_allowed_reference_types_at_publish(
    bt: &SignatureToken,
    module_bin_view: &BinaryIndexedView,
) -> bool {
    match bt {
        SignatureToken::Struct(sid) | SignatureToken::StructInstantiation(sid, _) => {
            let struct_full_name = struct_full_name_from_sid(sid, module_bin_view);
            is_allowed_input_struct(struct_full_name, true)
        }
        _ => false,
    }
}

fn struct_full_name_from_sid(
    sid: &StructHandleIndex,
    module_bin_view: &BinaryIndexedView,
) -> String {
    let struct_handle = module_bin_view.struct_handle_at(*sid);
    let struct_name = module_bin_view
        .identifier_at(struct_handle.name)
        .to_string();
    let module_name = module_bin_view
        .identifier_at(module_bin_view.module_handle_at(struct_handle.module).name)
        .to_string();
    let module_address = module_bin_view
        .address_identifier_at(
            module_bin_view
                .module_handle_at(struct_handle.module)
                .address,
        )
        .short_str_lossless();
    format!("0x{}::{}::{}", module_address, module_name, struct_name)
}

fn check_transaction_input_type<S>(ety: &Type, session: &Session<S>) -> bool
where
    S: TransactionCache,
{
    use Type::*;
    match ety {
        // Any primitive type allowed, any parameter expected to instantiate with primitive
        Bool | U8 | U16 | U32 | U64 | U128 | U256 | Address | Signer => true,
        Vector(ety) => {
            // Vectors are allowed if element type is allowed
            check_transaction_input_type(ety.deref(), session)
        }
        Struct(idx) | StructInstantiation(idx, _) => {
            if let Some(st) = session.get_struct_type(*idx) {
                let full_name = format!("{}::{}", st.module.short_str_lossless(), st.name);
                is_allowed_input_struct(full_name, false)
            } else {
                false
            }
        }
        Reference(bt)
            if matches!(bt.as_ref(), Signer)
                || is_allowed_reference_types(bt.as_ref(), session) =>
        {
            // Immutable Reference to signer and specific types is allowed
            true
        }
        MutableReference(bt) if is_allowed_reference_types(bt.as_ref(), session) => {
            // Mutable references to specific types is allowed
            true
        }
        _ => {
            // Everything else is disallowed.
            false
        }
    }
}

fn is_allowed_reference_types<S>(bt: &Type, session: &Session<S>) -> bool
where
    S: TransactionCache,
{
    match bt {
        Type::Struct(sid) | Type::StructInstantiation(sid, _) => {
            let st_option = session.get_struct_type(*sid);
            debug_assert!(
                st_option.is_some(),
                "Can not find by struct handle index:{:?}",
                sid
            );
            if let Some(st) = st_option {
                let full_name = format!("{}::{}", st.module.short_str_lossless(), st.name);
                is_allowed_input_struct(full_name, true)
            } else {
                false
            }
        }
        _ => false,
    }
}

fn vm_error_for_init_func_checking(
    status_code: StatusCode,
    error_message: &str,
    func_def: &FunctionDefinition,
    module_id: ModuleId,
) -> VMError {
    PartialVMError::new(status_code)
        .with_message(error_message.to_string())
        .at_code_offset(FunctionDefinitionIndex::new(func_def.function.0), 0_u16)
        .finish(Location::Module(module_id))
}

fn check_module_owner(item: &String, current_module: &CompiledModule) -> VMResult<bool> {
    let func_name_split = item.split("::");
    let parts_vec = func_name_split.collect::<Vec<&str>>();
    if (parts_vec.len() as u32) < 3 {
        return Err(PartialVMError::new(StatusCode::MALFORMED)
            .with_message("incorrect format of the item name in metadata".to_string())
            .finish(Location::Module(current_module.self_id())));
    }

    let module_address = parts_vec.first().unwrap();
    let module_name = parts_vec.get(1).unwrap();

    let current_module_address = current_module.address().to_hex_literal();
    let current_module_name = current_module.name().to_string();

    if *module_address != current_module_address.as_str()
        || *module_name != current_module_name.as_str()
    {
        return Err(PartialVMError::new(StatusCode::MALFORMED)
            .with_message(format!(
                "the metadata item {} is not belongs to {} module",
                item,
                current_module.self_id().to_string()
            ))
            .finish(Location::Module(current_module.self_id())));
    }
    Ok(true)
}

pub fn verify_private_generics<Resolver>(
    module: &CompiledModule,
    db: &Resolver,
    verified_modules: &mut BTreeMap<ModuleId, CompiledModule>,
) -> VMResult<bool>
where
    Resolver: ModuleResolver,
{
    if let Err(err) = check_metadata_format(module) {
        return Err(PartialVMError::new(StatusCode::MALFORMED)
            .with_message(err.to_string())
            .finish(Location::Module(module.self_id())));
    }

    let metadata_opt = get_metadata_from_compiled_module(module);
    match metadata_opt {
        None => {
            // If ROOCH_METADATA_KEY cannot be found in the metadata,
            // it means that the user's code did not use #[private_generics(T)],
            // or the user intentionally deleted the data in the metadata.
            // In either case, we will skip the verification.
            return Ok(true);
        }

        Some(metadata) => {
            let mut type_name_indices = metadata.private_generics_indices;

            for (full_func_name, _) in type_name_indices.iter() {
                check_module_owner(full_func_name, module)?;
                let (exists, _) = check_if_function_exist_in_module(module, full_func_name);
                if !exists {
                    return generate_vm_error(
                        StatusCode::RESOURCE_DOES_NOT_EXIST,
                        format!(
                            "Function {} not exist in module {}",
                            full_func_name,
                            module.self_id().to_string()
                        ),
                        None,
                        module,
                    );
                }
            }

            let view = BinaryIndexedView::Module(module);

            for func in &module.function_defs {
                if let Some(code_unit) = &func.code {
                    for instr in code_unit.code.clone().into_iter() {
                        if let Bytecode::CallGeneric(finst_idx) = instr {
                            // Find the module where a function is located based on its InstantiationIndex,
                            // and then find the metadata of the module.
                            let compiled_module_opt = load_compiled_module_from_finst_idx(
                                db,
                                &view,
                                finst_idx,
                                verified_modules,
                                true,
                            );

                            if let Some(compiled_module) = compiled_module_opt {
                                if let Err(err) = check_metadata_format(&compiled_module) {
                                    return Err(PartialVMError::new(StatusCode::MALFORMED)
                                        .with_message(err.to_string())
                                        .finish(Location::Module(compiled_module.self_id())));
                                }

                                // Find the definition records of compile-time private_generics from CompiledModule.
                                let metadata_opt =
                                    get_metadata_from_compiled_module(&compiled_module);
                                if let Some(metadata) = metadata_opt {
                                    let _ = metadata
                                        .private_generics_indices
                                        .iter()
                                        .map(|(key, value)| {
                                            type_name_indices.insert(key.clone(), value.clone())
                                        })
                                        .collect::<Vec<_>>();
                                }
                            }

                            let FunctionInstantiation {
                                handle,
                                type_parameters,
                            } = view.function_instantiation_at(finst_idx);

                            let full_path_func_name = build_full_function_name(handle, view);

                            let type_arguments = &view.signature_at(*type_parameters).0;
                            let private_generics_types =
                                type_name_indices.get(full_path_func_name.as_str());

                            if let Some(private_generics_types_indices) = private_generics_types {
                                for generic_type_index in private_generics_types_indices {
                                    let type_arg = match type_arguments.get(*generic_type_index) {
                                        None => {
                                            return generate_vm_error(
                                                StatusCode::RESOURCE_DOES_NOT_EXIST,
                                                format!("the function {} does not have enough type arguments.", full_path_func_name),
                                                None,
                                                module,
                                            );
                                        }
                                        Some(v) => v,
                                    };

                                    let (defined_in_current_module, struct_name) =
                                        is_defined_or_allowed_in_current_module(&view, type_arg);

                                    if !defined_in_current_module {
                                        let err_msg = format!(
                                            "resource type {:?} in function {:?} not defined in current module or not allowed",
                                            struct_name, full_path_func_name
                                        );

                                        return Err(PartialVMError::new(
                                            StatusCode::ABORT_TYPE_MISMATCH_ERROR,
                                        )
                                        .with_message(err_msg)
                                        .at_code_offset(
                                            FunctionDefinitionIndex::new(func.function.0),
                                            0_u16,
                                        )
                                        .finish(Location::Module(module.self_id())));
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(true)
}

fn build_full_function_name(fhandle_idx: &FunctionHandleIndex, view: BinaryIndexedView) -> String {
    let fhandle = view.function_handle_at(*fhandle_idx);
    let module_handle = view.module_handle_at(fhandle.module);

    let module_address = view
        .address_identifier_at(module_handle.address)
        .to_hex_literal();
    let module_name = view.identifier_at(module_handle.name);
    let func_name = view.identifier_at(fhandle.name).to_string();

    format!("{}::{}::{}", module_address, module_name, func_name)
}

pub fn verify_gas_free_function(module: &CompiledModule) -> VMResult<bool> {
    if let Err(err) = check_metadata_format(module) {
        return Err(PartialVMError::new(StatusCode::MALFORMED)
            .with_message(err.to_string())
            .finish(Location::Module(module.self_id())));
    }

    let metadata_opt = get_metadata_from_compiled_module(module);
    match metadata_opt {
        None => {
            // If ROOCH_METADATA_KEY cannot be found in the metadata,
            // it means that the user's code did not use #[private_generics(T)],
            // or the user intentionally deleted the data in the metadata.
            // In either case, we will skip the verification.
            return Ok(true);
        }

        Some(metadata) => {
            let gas_free_functions = metadata.gas_free_function_map;
            let view = BinaryIndexedView::Module(module);

            for (gas_free_function, gas_function_def) in gas_free_functions.iter() {
                // check the existence of the #[gas_free] function, if not we will return failed info.
                // The existence means that the #[gas_free] function must be defined in current module.
                let (func_exists, func_handle_index) =
                    check_if_function_exist_in_module(module, gas_free_function);

                if !func_exists {
                    let full_path_module_name = generate_full_module_name(func_handle_index, view);

                    let err_msg = format!(
                        "#[gas_free] function {:?} not defined in module {:?}",
                        gas_free_function, full_path_module_name
                    );

                    return generate_vm_error(
                        StatusCode::RESOURCE_DOES_NOT_EXIST,
                        err_msg,
                        Some(func_handle_index),
                        module,
                    );
                }

                // check the existence of the 'gas validate' function, if not we will return failed info.
                let gas_validate_function = gas_function_def.gas_validate.clone();
                let (func_exists, func_handle_index) =
                    check_if_function_exist_in_module(module, &gas_validate_function);
                if !func_exists {
                    let full_path_module_name = generate_full_module_name(func_handle_index, view);

                    let err_msg = format!(
                        "gas_validate function {:?} not defined in module {:?}",
                        gas_validate_function, full_path_module_name
                    );

                    return generate_vm_error(
                        StatusCode::RESOURCE_DOES_NOT_EXIST,
                        err_msg,
                        Some(func_handle_index),
                        module,
                    );
                }

                // check if the parameters and return types of the 'gas validate' function are legally.
                let func_handle = view.function_handle_at(func_handle_index);
                let func_parameters_index = func_handle.parameters;
                let func_signature = view.signature_at(func_parameters_index);
                let return_type_index = func_handle.return_;
                let return_signature = view.signature_at(return_type_index);

                if func_signature.is_empty() || return_signature.is_empty() {
                    let full_path_module_name = generate_full_module_name(func_handle_index, view);

                    let err_msg = format!(
                        "function {:?} in module {:?} with empty arguments or empty return value.",
                        gas_validate_function, full_path_module_name
                    );

                    return generate_vm_error(
                        StatusCode::RESOURCE_DOES_NOT_EXIST,
                        err_msg,
                        Some(func_handle_index),
                        module,
                    );
                }

                if func_signature.len() != 1 && return_signature.len() != 1 {
                    let full_path_module_name = generate_full_module_name(func_handle_index, view);

                    let err_msg = format!(
                            "function {:?} in module {:?} with incorrect number of parameters or return values.",
                            gas_validate_function, full_path_module_name
                        );

                    return generate_vm_error(
                        StatusCode::TOO_MANY_PARAMETERS,
                        err_msg,
                        Some(func_handle_index),
                        module,
                    );
                }

                let parameter_allowed =
                    check_gas_validate_function(&view, func_signature, return_signature);
                if !parameter_allowed {
                    let full_path_module_name = generate_full_module_name(func_handle_index, view);

                    let err_msg = format!(
                        "gas validate function {:?} in module {:?} has incorrect parameter type or return type.",
                        gas_validate_function, full_path_module_name
                    );
                    return generate_vm_error(
                        StatusCode::TYPE_MISMATCH,
                        err_msg,
                        Some(func_handle_index),
                        module,
                    );
                }

                // check the existence of the 'gas charge post' function, if not we will return failed info.
                // check if the parameters and return types of the 'gas charge' function are legally.
                let gas_charge_post_function = gas_function_def.gas_charge_post.clone();
                let (func_exists, func_handle_index) =
                    check_if_function_exist_in_module(module, &gas_charge_post_function);
                if !func_exists {
                    let full_path_module_name = generate_full_module_name(func_handle_index, view);

                    let err_msg = format!(
                        "gas_validate function {:?} not defined in module {:?}",
                        gas_charge_post_function, full_path_module_name
                    );

                    return generate_vm_error(
                        StatusCode::RESOURCE_DOES_NOT_EXIST,
                        err_msg,
                        Some(func_handle_index),
                        module,
                    );
                }

                // check if the parameters and return types of the 'gas validate' function are legally.
                let func_handle = view.function_handle_at(func_handle_index);
                let func_parameters_index = func_handle.parameters;
                let func_signature = view.signature_at(func_parameters_index);
                let return_type_index = func_handle.return_;
                let return_signature = view.signature_at(return_type_index);

                if func_signature.is_empty() || return_signature.is_empty() {
                    let full_path_module_name = generate_full_module_name(func_handle_index, view);

                    let err_msg = format!(
                        "function {:?} in module {:?} with empty arguments or empty return value.",
                        gas_validate_function, full_path_module_name
                    );

                    return generate_vm_error(
                        StatusCode::RESOURCE_DOES_NOT_EXIST,
                        err_msg,
                        Some(func_handle_index),
                        module,
                    );
                }

                if func_signature.len() != 2 || return_signature.len() != 1 {
                    let full_path_module_name = generate_full_module_name(func_handle_index, view);

                    let err_msg = format!(
                        "function {:?} in module {:?} with incorrect number of parameters or return values.",
                        gas_validate_function, full_path_module_name
                    );

                    return generate_vm_error(
                        StatusCode::TOO_MANY_PARAMETERS,
                        err_msg,
                        Some(func_handle_index),
                        module,
                    );
                }

                let parameter_allowed =
                    check_gas_charge_post_function(&view, func_signature, return_signature);

                if !parameter_allowed {
                    let full_path_module_name = generate_full_module_name(func_handle_index, view);

                    let err_msg = format!(
                        "function {:?} in module {:?} has incorrect parameter type or return type.",
                        gas_validate_function, full_path_module_name
                    );
                    return generate_vm_error(
                        StatusCode::TYPE_MISMATCH,
                        err_msg,
                        Some(func_handle_index),
                        module,
                    );
                }
            }
        }
    }

    Ok(true)
}

pub fn verify_data_struct<Resolver>(
    caller_module: &CompiledModule,
    db: &Resolver,
    verified_modules: &mut BTreeMap<ModuleId, CompiledModule>,
) -> VMResult<bool>
where
    Resolver: ModuleResolver,
{
    if let Err(err) = check_metadata_format(caller_module) {
        return Err(PartialVMError::new(StatusCode::MALFORMED)
            .with_message(err.to_string())
            .finish(Location::Module(caller_module.self_id())));
    }

    let metadata_opt = get_metadata_from_compiled_module(caller_module);
    match metadata_opt {
        None => {
            // If ROOCH_METADATA_KEY cannot be found in the metadata,
            // it means that the user's code did not use #[data_struct(T)],
            // or the user intentionally deleted the data in the metadata.
            // In either case, we will skip the verification.
            return Ok(true);
        }

        Some(metadata) => {
            let mut data_structs_map = metadata.data_struct_map;
            let mut data_structs_func_map = metadata.data_struct_func_map;

            for (full_struct_name, _) in data_structs_map.iter() {
                check_module_owner(full_struct_name, caller_module)?;
                let exists = check_if_struct_exist_in_module(caller_module, full_struct_name);
                if !exists {
                    return generate_vm_error(
                        StatusCode::RESOURCE_DOES_NOT_EXIST,
                        format!(
                            "Struct {} not exist in module {}",
                            full_struct_name,
                            caller_module.self_id().to_string()
                        ),
                        None,
                        caller_module,
                    );
                }
            }

            for (full_func_name, _) in data_structs_func_map.iter() {
                check_module_owner(full_func_name, caller_module)?;
                let (exists, _) = check_if_function_exist_in_module(caller_module, full_func_name);
                if !exists {
                    return generate_vm_error(
                        StatusCode::RESOURCE_DOES_NOT_EXIST,
                        format!(
                            "Function {} not exist in module {}",
                            full_func_name,
                            caller_module.self_id().to_string()
                        ),
                        None,
                        caller_module,
                    );
                }
            }

            let view = BinaryIndexedView::Module(caller_module);

            for func in &caller_module.function_defs {
                if let Some(code_unit) = &func.code {
                    for instr in code_unit.code.clone().into_iter() {
                        if let Bytecode::CallGeneric(finst_idx) = instr {
                            // Find the module where a function is located based on its InstantiationIndex,
                            // and then find the metadata of the module.
                            let compiled_module_opt = load_compiled_module_from_finst_idx(
                                db,
                                &view,
                                finst_idx,
                                verified_modules,
                                true,
                            );

                            if let Some(callee_module) = compiled_module_opt {
                                if let Err(err) = check_metadata_format(&callee_module) {
                                    return Err(PartialVMError::new(StatusCode::MALFORMED)
                                        .with_message(err.to_string())
                                        .finish(Location::Module(callee_module.self_id())));
                                }

                                // Find the definition records of compile-time data_struct from CompiledModule.
                                let metadata_opt =
                                    get_metadata_from_compiled_module(&callee_module);
                                if let Some(metadata) = metadata_opt {
                                    let _ = metadata
                                        .data_struct_func_map
                                        .iter()
                                        .map(|(key, value)| {
                                            data_structs_func_map.insert(key.clone(), value.clone())
                                        })
                                        .collect::<Vec<_>>();
                                }
                            }

                            let FunctionInstantiation {
                                handle: fhandle_idx,
                                type_parameters,
                            } = view.function_instantiation_at(finst_idx);

                            let fhandle = view.function_handle_at(*fhandle_idx);
                            let module_handle = view.module_handle_at(fhandle.module);

                            let module_address = view
                                .address_identifier_at(module_handle.address)
                                .to_hex_literal();
                            let module_name = view.identifier_at(module_handle.name);
                            let func_name = view.identifier_at(fhandle.name).to_string();

                            // The function name which the CallGeneric is called.
                            let full_path_func_name =
                                format!("{}::{}::{}", module_address, module_name, func_name);

                            let type_arguments = &view.signature_at(*type_parameters).0;
                            let data_struct_func_types =
                                data_structs_func_map.get(full_path_func_name.as_str());

                            if let Some(data_struct_types_indices) = data_struct_func_types {
                                for generic_type_index in data_struct_types_indices {
                                    let type_arg = match type_arguments.get(*generic_type_index) {
                                        None => {
                                            return generate_vm_error(
                                                StatusCode::RESOURCE_DOES_NOT_EXIST,
                                                format!("the function {} does not have enough type arguments.", full_path_func_name),
                                                None,
                                                caller_module,
                                            );
                                        }
                                        Some(v) => v,
                                    };

                                    match type_arg {
                                        SignatureToken::Struct(struct_handle_idx) => {
                                            let struct_handle =
                                                view.struct_handle_at(*struct_handle_idx);
                                            let module_handle =
                                                view.module_handle_at(struct_handle.module);
                                            let module_name = format!(
                                                "{}::{}",
                                                view.address_identifier_at(module_handle.address)
                                                    .to_hex_literal(),
                                                view.identifier_at(module_handle.name),
                                            );

                                            // load module from struct handle
                                            let compiled_module_opt =
                                                load_compiled_module_from_struct_handle(
                                                    db,
                                                    &view,
                                                    *struct_handle_idx,
                                                    verified_modules,
                                                );
                                            if let Some(callee_module) = compiled_module_opt {
                                                if let Err(err) =
                                                    check_metadata_format(&callee_module)
                                                {
                                                    return Err(PartialVMError::new(
                                                        StatusCode::MALFORMED,
                                                    )
                                                    .with_message(err.to_string())
                                                    .finish(Location::Module(
                                                        callee_module.self_id(),
                                                    )));
                                                }

                                                // Find the definition records of compile-time data_struct from CompiledModule.
                                                let metadata_opt =
                                                    get_metadata_from_compiled_module(
                                                        &callee_module,
                                                    );
                                                if let Some(metadata) = metadata_opt {
                                                    let _ = metadata
                                                        .data_struct_map
                                                        .iter()
                                                        .map(|(key, value)| {
                                                            data_structs_map
                                                                .insert(key.clone(), *value);
                                                        })
                                                        .collect::<Vec<_>>();
                                                }
                                            }

                                            let full_struct_name = format!(
                                                "{}::{}",
                                                module_name,
                                                view.identifier_at(struct_handle.name)
                                            );
                                            let is_data_struct_opt =
                                                data_structs_map.get(full_struct_name.as_str());
                                            if is_data_struct_opt.is_none() {
                                                let caller_func_name =
                                                    build_full_function_name(&func.function, view);
                                                let error_msg = format!("function {:} call {:} with type {:} is not a data struct.",
                                                                        caller_func_name, full_path_func_name, full_struct_name);
                                                return generate_vm_error(
                                                    StatusCode::TYPE_MISMATCH,
                                                    error_msg,
                                                    Some(*fhandle_idx),
                                                    caller_module,
                                                );
                                            }
                                        }
                                        SignatureToken::Address => {}
                                        SignatureToken::Bool => {}
                                        SignatureToken::U8 => {}
                                        SignatureToken::U16 => {}
                                        SignatureToken::U32 => {}
                                        SignatureToken::U64 => {}
                                        SignatureToken::U128 => {}
                                        SignatureToken::U256 => {}
                                        _ => {
                                            let error_msg = format!("The type parameter when calling function {} is now allowed",
                                                                    full_path_func_name);
                                            return generate_vm_error(
                                                StatusCode::TYPE_MISMATCH,
                                                error_msg,
                                                Some(*fhandle_idx),
                                                caller_module,
                                            );
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    verified_modules.insert(caller_module.self_id(), caller_module.clone());
    Ok(true)
}

fn generate_full_module_name(
    fhandle_index: FunctionHandleIndex,
    view: BinaryIndexedView,
) -> String {
    let fhandle = view.function_handle_at(fhandle_index);
    let module_handle = view.module_handle_at(fhandle.module);

    let module_address = view
        .address_identifier_at(module_handle.address)
        .to_hex_literal();
    let module_name = view.identifier_at(module_handle.name);
    format!("{}::{}", module_address, module_name)
}

pub fn generate_vm_error(
    status_code: StatusCode,
    error_msg: String,
    fhandle: Option<FunctionHandleIndex>,
    module: &CompiledModule,
) -> VMResult<bool> {
    let err_incomplete = PartialVMError::new(status_code).with_message(error_msg);
    let fdef_idx = fhandle.unwrap_or_else(|| FunctionHandleIndex::new(0));
    Err(err_incomplete
        .at_code_offset(FunctionDefinitionIndex::new(fdef_idx.0), 0_u16)
        .finish(Location::Module(module.self_id())))
}

fn check_if_struct_exist_in_module(module: &CompiledModule, origin_struct_name: &String) -> bool {
    let module_bin_view = BinaryIndexedView::Module(module);
    for struct_def in module.struct_defs.iter() {
        let module_address = module.address().to_hex_literal();
        let module_name = module.name().to_string();
        let struct_handle = module_bin_view.struct_handle_at(struct_def.struct_handle);
        let struct_name = module_bin_view
            .identifier_at(struct_handle.name)
            .to_string();
        let full_struct_name = format!("{}::{}::{}", module_address, module_name, struct_name);
        if full_struct_name == *origin_struct_name {
            return true;
        }
    }
    false
}

fn check_if_function_exist_in_module(
    module: &CompiledModule,
    function_name: &String,
) -> (bool, FunctionHandleIndex) {
    let module_bin_view = BinaryIndexedView::Module(module);
    for fdef in module.function_defs.iter() {
        let func_handle = module_bin_view.function_handle_at(fdef.function);
        let module_address = module.address().to_hex_literal();
        let module_name = module.name().to_string();
        let func_name = module_bin_view.identifier_at(func_handle.name).to_string();
        let full_func_name = format!("{}::{}::{}", module_address, module_name, func_name);
        if &full_func_name == function_name {
            let fhandle_index = fdef.function.0;
            return (true, FunctionHandleIndex::new(fhandle_index));
        }
    }

    (false, FunctionHandleIndex::new(0))
}

fn check_gas_validate_function(
    _view: &BinaryIndexedView,
    func_signature: &Signature,
    return_signature: &Signature,
) -> bool {
    // Content of the func_signature array has already been checked above, so unwrap directly here.
    let first_parameter = func_signature.0.first().unwrap();

    let check_struct_type = |_struct_handle_idx: &StructHandleIndex| -> bool {
        //TODO FIXME
        false
    };

    let parameter_check_result = match first_parameter {
        SignatureToken::Reference(reference) => match reference.as_ref() {
            SignatureToken::Struct(struct_handle_idx) => check_struct_type(struct_handle_idx),
            _ => false,
        },
        SignatureToken::Struct(struct_handle_idx) => check_struct_type(struct_handle_idx),
        _ => false,
    };

    if !parameter_check_result {
        return parameter_check_result;
    }

    if return_signature.len() != 1 {
        return false;
    }

    // Content of the return_signature array has already been checked above, so unwrap directly here.
    let first_return_signature = return_signature.0.first().unwrap();
    matches!(first_return_signature, SignatureToken::Bool)
}

fn check_gas_charge_post_function(
    _view: &BinaryIndexedView,
    func_signature: &Signature,
    return_signature: &Signature,
) -> bool {
    // Content of the func_signature array has already been checked above, so unwrap directly here.
    let first_parameter = func_signature.0.first().unwrap();

    let check_struct_type = |_struct_handle_idx: &StructHandleIndex| -> bool {
        //TODO FIXME
        false
    };

    let first_checking_result = {
        match first_parameter {
            SignatureToken::MutableReference(reference) => match reference.as_ref() {
                SignatureToken::Struct(struct_handle_idx) => check_struct_type(struct_handle_idx),
                _ => false,
            },
            SignatureToken::Struct(struct_handle_idx) => check_struct_type(struct_handle_idx),
            _ => false,
        }
    };

    if !first_checking_result {
        return first_checking_result;
    }

    // Content of the func_signature array has already been checked above, so unwrap directly here.
    let second_parameter = func_signature.0.get(1).unwrap();
    let second_checking_result = matches!(second_parameter, SignatureToken::U128);

    if !second_checking_result {
        return second_checking_result;
    }

    if return_signature.len() != 1 {
        return false;
    }

    // Content of the return_signature array has already been checked above, so unwrap directly here.
    let first_return_signature = return_signature.0.first().unwrap();
    matches!(first_return_signature, SignatureToken::Bool)
}

fn load_compiled_module_from_struct_handle<Resolver>(
    db: &Resolver,
    view: &BinaryIndexedView,
    struct_idx: StructHandleIndex,
    verified_modules: &mut BTreeMap<ModuleId, CompiledModule>,
) -> Option<CompiledModule>
where
    Resolver: ModuleResolver,
{
    let struct_handle = view.struct_handle_at(struct_idx);
    let module_handle = view.module_handle_at(struct_handle.module);
    let module_address = view.address_identifier_at(module_handle.address);
    let module_name = view.identifier_at(module_handle.name);
    let module_id = ModuleId::new(*module_address, Identifier::from(module_name));

    match verified_modules.get(&module_id) {
        None => get_module_from_db(&module_id, db),
        Some(m) => Some(m.clone()),
    }
}

// Find the module where a function is located based on its InstantiationIndex.
fn load_compiled_module_from_finst_idx<Resolver>(
    db: &Resolver,
    view: &BinaryIndexedView,
    finst_idx: FunctionInstantiationIndex,
    verified_modules: &mut BTreeMap<ModuleId, CompiledModule>,
    search_verified_modules: bool,
) -> Option<CompiledModule>
where
    Resolver: ModuleResolver,
{
    let FunctionInstantiation {
        handle,
        type_parameters: _type_parameters,
    } = view.function_instantiation_at(finst_idx);

    let fhandle = view.function_handle_at(*handle);
    let module_handle = view.module_handle_at(fhandle.module);

    let module_address = view.address_identifier_at(module_handle.address);
    let module_name = view.identifier_at(module_handle.name);
    let module_id = ModuleId::new(*module_address, Identifier::from(module_name));
    if search_verified_modules {
        match verified_modules.get(&module_id) {
            None => get_module_from_db(&module_id, db),
            Some(m) => Some(m.clone()),
        }
    } else {
        get_module_from_db(&module_id, db)
    }
}

fn get_module_from_db<Resolver>(module_id: &ModuleId, db: &Resolver) -> Option<CompiledModule>
where
    Resolver: ModuleResolver,
{
    match db.get_module(module_id) {
        Err(_) => None,
        Ok(value) => match value {
            None => None,
            Some(bytes) => CompiledModule::deserialize(bytes.as_slice()).ok(),
        },
    }
}

pub fn verify_global_storage_access(module: &CompiledModule) -> VMResult<bool> {
    let view = BinaryIndexedView::Module(module);

    for func in &module.function_defs {
        let mut invalid_bytecode = vec![];
        if let Some(func_code) = func.clone().code {
            for instr in func_code.code {
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
        }

        if !invalid_bytecode.is_empty() {
            let fhandle = view.function_handle_at(func.function);
            let func_name = view.identifier_at(fhandle.name).to_string();

            let error_msg = format!(
                "Access to Move global storage is not allowed. Found in function {}: {:?}",
                func_name, invalid_bytecode,
            );

            return Err(PartialVMError::new(StatusCode::MALFORMED)
                .with_message(error_msg)
                .at_code_offset(FunctionDefinitionIndex::new(func.function.0), 0_u16)
                .finish(Location::Module(module.self_id())));
        }
    }
    Ok(true)
}
