use crate::metadata::is_allowed_input_struct;
use anyhow::{Error, Ok, Result};
use move_binary_format::file_format::Visibility;
use move_binary_format::{access::ModuleAccess, CompiledModule};
use move_core_types::move_resource::MoveStructType;
use move_core_types::{identifier::Identifier, resolver::MoveResolver};
use move_vm_runtime::session::{LoadedFunctionInstantiation, Session};
use move_vm_types::loaded_data::runtime_types::{StructType, Type};
use moveos_types::move_types::FunctionId;
use moveos_types::storage_context::StorageContext;
use once_cell::sync::Lazy;
use std::ops::Deref;
use std::sync::Arc;

pub static INIT_FN_NAME_IDENTIFIER: Lazy<Identifier> =
    Lazy::new(|| Identifier::new("init").unwrap());

fn as_struct_no_panic<S>(session: &Session<S>, t: &Type) -> Option<Arc<StructType>>
where
    S: MoveResolver,
{
    match t {
        Type::Struct(s) | Type::StructInstantiation(s, _) => session.get_struct_type(*s),
        Type::Reference(r) => as_struct_no_panic(session, r),
        Type::MutableReference(r) => as_struct_no_panic(session, r),
        _ => None,
    }
}

fn is_storage_context(t: &StructType) -> bool {
    *t.module.address() == *moveos_types::addresses::MOVEOS_STD_ADDRESS
        && t.module.name() == StorageContext::module_identifier().as_ident_str()
        && t.name == StorageContext::struct_identifier()
}

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
            } else if !fdef.is_entry {
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
                let Some((_i, _t)) = loaded_function.parameters.iter().enumerate().find(|(i, t)| {
                        let struct_type = as_struct_no_panic(session, t);
                        (*i as u32 == 0u32) && Option::is_some(&struct_type) && is_storage_context(&(struct_type.unwrap()))
                    }) else {
                        return Ok(true)
                    };

                if !(loaded_function.return_.is_empty()) {
                    return Ok(true);
                }
            }
        }
    }
    Err(Error::msg("module not have init function".to_string()))
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
