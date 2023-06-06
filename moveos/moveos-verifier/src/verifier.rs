use crate::metadata::{check_storage_context_struct_tag, is_allowed_input_struct};
use anyhow::{Error, Ok, Result};
use move_binary_format::file_format::Visibility;
use move_binary_format::{access::ModuleAccess, CompiledModule};
use move_core_types::{identifier::Identifier, resolver::MoveResolver};
use move_vm_runtime::session::{LoadedFunctionInstantiation, Session};
use move_vm_types::loaded_data::runtime_types::Type;
use moveos_types::move_types::FunctionId;
use once_cell::sync::Lazy;
use std::ops::Deref;

pub static INIT_FN_NAME_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("init").unwrap());

/// The initializer function must have the following properties in order to be executed at publication:
/// - Name init
/// - Single parameter of &mut TxContext type
/// - No return values
/// - Private
pub fn verify_init_function<S>(module: &CompiledModule, session: &Session<S>) -> Result<bool>
where
    S: MoveResolver,
{
    for fdef in &module.function_defs {
        let fhandle = module.function_handle_at(fdef.function);
        let fname = module.identifier_at(fhandle.name);
        if fname == INIT_FN_NAME_IDENTIFIER.clone().as_ident_str() {
            if Visibility::Private != fdef.visibility {
                return Err(Error::msg("init function should private".to_string()));
            } else if fdef.is_entry {
                return Err(Error::msg(
                    "init function should not entry function".to_string(),
                ));
            } else {
                let function_id =
                    FunctionId::new(module.self_id(), INIT_FN_NAME_IDENTIFIER.clone());
                let loaded_function = session.load_function(
                    &module.self_id(),
                    &function_id.function_name,
                    vec![].as_slice(),
                )?;
                let parameters_usize = loaded_function.parameters.len();
                if parameters_usize != 1 && parameters_usize != 2 {
                    return Err(Error::msg(
                        "init function only should have two parameter with signer or storageContext"
                            .to_string(),
                    ));
                }
                for ref ty in loaded_function.parameters {
                    match ty {
                        Type::Reference( bt) | Type::MutableReference(bt)=> {
                            match bt.as_ref() {
                                Type::Struct(s) | Type::StructInstantiation(s, _) => {
                                    let struct_type = session.get_struct_type(*s).unwrap();
                                    if !check_storage_context_struct_tag(
                                        &(struct_type.module.address().to_string() + "::"
                                            + &struct_type.module.name().to_string() + "::"
                                            + &struct_type.name.to_string()),
                                    ) {
                                        return Err(Error::msg(
                                            "init function should not input structures other than storageContext"
                                                .to_string(),
                                        ))
                                    }
                                }
                                Type::Signer => {}
                                _ => {
                                    return Err(Error::msg(
                                        "init function should only enter reference signer or mutable reference storageContext"
                                            .to_string(),
                                    ))
                                }
                            }
                        }
                        Type::Signer => {}
                        _ => return Err(Error::msg(
                            "init function should only enter signer or storageContext"
                                .to_string(),
                        ))
                    }
                }
            }
        }
    }
    Ok(false)
}

pub fn verify_entry_function<S>(
    func: LoadedFunctionInstantiation,
    session: &Session<S>,
) -> Result<bool>
where
    S: MoveResolver,
{
    if !func.return_.is_empty() {
        return Err(Error::msg("function should not return values".to_string()));
    }

    for ty in &func.parameters {
        if !check_transaction_input_type(ty, session) {
            return Err(Error::msg("parameter type is not allowed".to_string()));
        }
    }

    Ok(true)
}

fn check_transaction_input_type<S>(ety: &Type, session: &Session<S>) -> bool
where
    S: MoveResolver,
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
                is_allowed_input_struct(full_name)
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
    S: MoveResolver,
{
    match bt {
        Type::Struct(sid) => {
            if let Some(st) = session.get_struct_type(*sid) {
                let full_name = format!("{}::{}", st.module.short_str_lossless(), st.name);
                if is_allowed_input_struct(full_name) {
                    return true;
                }
            }

            false
        }
        _ => false,
    }
}
