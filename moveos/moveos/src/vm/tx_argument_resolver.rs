// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_binary_format::errors::PartialVMError;
use move_core_types::{move_resource::MoveStructType, value::MoveValue, vm_status::StatusCode};
use move_vm_runtime::session::LoadedFunctionInstantiation;
use move_vm_types::loaded_data::runtime_types::{StructType, Type};
use moveos_stdlib::natives::moveos_stdlib::object_extension::NativeObjectContext;
use moveos_types::{
    object::{self, Object, ObjectID},
    tx_context::TxContext,
};
use std::sync::Arc;

use super::{move_vm_ext::SessionExt, MoveResolverExt};

/// Transaction Argument Resolver will implemented by the Move Extension
/// to auto fill transaction argument or do type conversion.
// TODO: Try to push to Move upstream if possible.
pub trait TxArgumentResolver {
    fn resolve_argument<S>(
        &self,
        session: &SessionExt<S>,
        func: &LoadedFunctionInstantiation,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>, PartialVMError>
    where
        S: MoveResolverExt;
}

impl TxArgumentResolver for TxContext {
    fn resolve_argument<S>(
        &self,
        session: &SessionExt<S>,
        func: &LoadedFunctionInstantiation,
        mut args: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>, PartialVMError>
    where
        S: MoveResolverExt,
    {
        let has_signer = func
            .parameters
            .iter()
            .position(is_signer)
            .map(|pos| {
                if pos != 0 {
                    Err(
                        PartialVMError::new(StatusCode::NUMBER_OF_SIGNER_ARGUMENTS_MISMATCH)
                            .with_message(format!(
                                "Expected signer arg is this first arg, but got it at {}",
                                pos + 1
                            )),
                    )
                } else {
                    Ok(true)
                }
            })
            .unwrap_or(Ok(false))?;

        if has_signer {
            let signer = MoveValue::Signer(self.sender());
            args.insert(
                0,
                signer
                    .simple_serialize()
                    .expect("serialize signer should success"),
            );
        }

        let has_tx_context = func
            .parameters
            .iter()
            .position(|i| {
                as_struct(session, i)
                    .map(|t| is_tx_context(&t))
                    .unwrap_or(false)
            })
            .map(|pos| {
                if pos != 0 {
                    Err(
                        PartialVMError::new(StatusCode::NUMBER_OF_SIGNER_ARGUMENTS_MISMATCH)
                            .with_message(format!(
                                "Expected TxContext arg is this first arg, but got it at {}",
                                pos + 1
                            )),
                    )
                } else {
                    Ok(true)
                }
            })
            .unwrap_or(Ok(false))?;

        if has_tx_context {
            args.insert(0, self.to_vec());
        }

        Ok(args)
    }
}

// TODO move to move_table_extension
impl TxArgumentResolver for move_table_extension::NativeTableContext<'_> {
    fn resolve_argument<S>(
        &self,
        _session: &SessionExt<S>,
        _func: &LoadedFunctionInstantiation,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>, PartialVMError>
    where
        S: MoveResolverExt,
    {
        //TableContext do nothing to the arguments now.
        Ok(args)
    }
}

//TODO move to NativeObjectContext
impl TxArgumentResolver for NativeObjectContext<'_> {
    fn resolve_argument<S>(
        &self,
        session: &SessionExt<S>,
        func: &LoadedFunctionInstantiation,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>, PartialVMError>
    where
        S: MoveResolverExt,
    {
        if args.len() != func.parameters.len() {
            return Err(PartialVMError::new(
                StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH,
            ));
        }
        let mut resolved_args = vec![];
        for (i, (t, arg)) in func.parameters.iter().zip(args.iter()).enumerate() {
            let struct_type = as_struct(session, t);
            let resolved_arg = match struct_type {
                Some(t) => {
                    //skip tx context and signer
                    if is_tx_context(&t) {
                        arg.clone()
                    } else if is_object(&t) {
                        let object_id = ObjectID::from_bytes(arg).map_err(|_e| {
                            PartialVMError::new(StatusCode::UNEXPECTED_DESERIALIZATION_ERROR)
                                .with_message("decode ObjectID error".to_string())
                        })?;
                        let object: Object = self.get_object(object_id)?.ok_or_else(|| {
                            PartialVMError::new(StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH)
                                .with_message(format!(
                                    "Can not find object: {:?} from arg:{:?}",
                                    object_id, i
                                ))
                        })?;
                        //object.as_move_resource default support table as argument,
                        //should we support table as argument?
                        object.as_object_argument(object_id).map_err(|_e| {
                            PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                                .with_message("encode Object argument error".to_string())
                        })?
                    } else {
                        return Err(PartialVMError::new(StatusCode::UNKNOWN_VALIDATION_STATUS)
                            .with_message("Unsupported Object argument type".to_string()));
                    }
                }
                None => arg.clone(),
            };

            resolved_args.push(resolved_arg);
        }

        Ok(resolved_args)
    }
}

fn is_signer(t: &Type) -> bool {
    matches!(t, Type::Signer)
}

fn as_struct<T>(session: &SessionExt<T>, t: &Type) -> Option<Arc<StructType>>
where
    T: MoveResolverExt,
{
    match t {
        Type::Struct(s) | Type::StructInstantiation(s, _) => match session.get_struct_type(*s) {
            Some(t) => Some(t),
            None => {
                panic!("Can not find type for struct: {:?}", s)
            }
        },
        Type::Reference(r) => as_struct(session, r),
        Type::MutableReference(r) => as_struct(session, r),
        _ => None,
    }
}

fn is_object(t: &StructType) -> bool {
    *t.module.address() == *moveos_stdlib::addresses::MOVEOS_STD_ADDRESS
        && t.module.name() == object::OBJECT_MODULE_NAME
        && t.name.as_ident_str() == object::OBJECT_STRUCT_NAME
}

fn is_tx_context(t: &StructType) -> bool {
    *t.module.address() == *moveos_stdlib::addresses::MOVEOS_STD_ADDRESS
        && t.module.name() == TxContext::module_identifier().as_ident_str()
        && t.name == TxContext::struct_identifier()
}
