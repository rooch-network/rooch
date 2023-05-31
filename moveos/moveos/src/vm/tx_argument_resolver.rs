// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::{move_vm_ext::SessionExt, MoveResolverExt};
use anyhow::Result;
use move_binary_format::errors::PartialVMError;
use move_core_types::{move_resource::MoveStructType, value::MoveValue};
use move_vm_runtime::session::LoadedFunctionInstantiation;
use move_vm_types::loaded_data::runtime_types::{StructType, Type};
use moveos_stdlib::natives::moveos_stdlib::raw_table::NativeTableContext;
use moveos_types::{storage_context::StorageContext, tx_context::TxContext};
use std::sync::Arc;

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
        func.parameters.iter().enumerate().for_each(|(i, t)| {
            if is_signer(t) {
                let signer = MoveValue::Signer(self.sender());
                args.insert(
                    i,
                    signer
                        .simple_serialize()
                        .expect("serialize signer should success"),
                );
            }
            if as_struct(session, t)
                .map(|t| is_tx_context(&t))
                .unwrap_or(false)
            {
                args.insert(i, self.to_vec());
            }

            if as_struct(session, t)
                .map(|t| is_storage_context(&t))
                .unwrap_or(false)
            {
                let storage_context = StorageContext::new(self.clone());
                args.insert(i, storage_context.to_vec());
            }
        });
        Ok(args)
    }
}

impl TxArgumentResolver for NativeTableContext<'_> {
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

fn is_signer(t: &Type) -> bool {
    matches!(t, Type::Signer) || matches!(t, Type::Reference(r) if matches!(**r, Type::Signer))
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

pub fn as_struct_no_panic<T>(session: &SessionExt<T>, t: &Type) -> Option<Arc<StructType>>
where
    T: MoveResolverExt,
{
    match t {
        Type::Struct(s) | Type::StructInstantiation(s, _) => session.get_struct_type(*s),
        Type::Reference(r) => as_struct_no_panic(session, r),
        Type::MutableReference(r) => as_struct_no_panic(session, r),
        _ => None,
    }
}

fn is_tx_context(t: &StructType) -> bool {
    *t.module.address() == *moveos_types::addresses::MOVEOS_STD_ADDRESS
        && t.module.name() == TxContext::module_identifier().as_ident_str()
        && t.name == TxContext::struct_identifier()
}

pub fn is_storage_context(t: &StructType) -> bool {
    *t.module.address() == *moveos_types::addresses::MOVEOS_STD_ADDRESS
        && t.module.name() == StorageContext::module_identifier().as_ident_str()
        && t.name == StorageContext::struct_identifier()
}
