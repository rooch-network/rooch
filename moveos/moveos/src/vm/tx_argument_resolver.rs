// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use framework::natives::mos_stdlib::object_extension::NativeObjectContext;
use moveos_types::{
    object::{Object, ObjectData, ObjectID},
    tx_context::TxContext,
};
use move_binary_format::errors::PartialVMError;
use move_core_types::{move_resource::MoveStructType, value::MoveValue, vm_status::StatusCode};
use move_vm_runtime::session::LoadedFunctionInstantiation;
use move_vm_types::loaded_data::runtime_types::{StructType, Type};
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
                is_object(session, i)
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
            let is_object = is_object(session, t);
            let arg = match is_object {
                Some(t) => {
                    //skip tx context and signer
                    if is_tx_context(&t) {
                        arg.clone()
                    } else {
                        let object_id = ObjectID::from_bytes(arg).map_err(|_e| {
                            PartialVMError::new(StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH)
                        })?;
                        let object: Object = self.get_object(object_id)?.ok_or_else(|| {
                            PartialVMError::new(StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH)
                                .with_message(format!("Can not find object: {:?}", object_id))
                        })?;
                        match object.data {
                            ObjectData::MoveObject(m) => m.contents,
                            ObjectData::TableObject(_) => {
                                return Err(PartialVMError::new(
                                    StatusCode::NUMBER_OF_TYPE_ARGUMENTS_MISMATCH,
                                )
                                .with_message(format!(
                            "Table object is not supported as argument currently, argument pos: {}",
                            i + 1
                        )))
                            }
                        }
                    }
                }
                None => arg.clone(),
            };
            resolved_args.push(arg);
        }

        Ok(args)
    }
}

fn is_signer(t: &Type) -> bool {
    matches!(t, Type::Signer)
}

fn is_object<T>(session: &SessionExt<T>, t: &Type) -> Option<Arc<StructType>>
where
    T: MoveResolverExt,
{
    match t {
        Type::Struct(s) => match session.get_struct_type(*s) {
            Some(t) => Some(t),
            None => {
                panic!("Can not find bype for struct: {:?}", s)
            }
        },
        Type::Reference(r) => is_object(session, r),
        Type::MutableReference(r) => is_object(session, r),
        _ => None,
    }
}

fn is_tx_context(t: &StructType) -> bool {
    *t.module.address() == *framework::addresses::MOS_STD_ADDRESS
        && t.module.name() == TxContext::module_identifier().as_ident_str()
        && t.name == TxContext::struct_identifier()
}
