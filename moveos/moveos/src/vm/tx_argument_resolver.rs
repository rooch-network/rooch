// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use anyhow::Result;
use move_binary_format::errors::PartialVMError;
use move_core_types::value::MoveValue;
use move_vm_runtime::session::{LoadedFunctionInstantiation, Session};
use move_vm_types::loaded_data::runtime_types::{StructType, Type};
use moveos_types::{
    state::MoveStructType, state_resolver::MoveOSResolver, storage_context::StorageContext,
    tx_context::TxContext,
};
use std::sync::Arc;

/// Transaction Argument Resolver will implemented by the Move Extension
/// to auto fill transaction argument or do type conversion.
pub trait TxArgumentResolver {
    fn resolve_argument<S>(
        &self,
        session: &Session<S>,
        func: &LoadedFunctionInstantiation,
        args: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>, PartialVMError>
    where
        S: MoveOSResolver;
}

impl TxArgumentResolver for StorageContext {
    fn resolve_argument<S>(
        &self,
        session: &Session<S>,
        func: &LoadedFunctionInstantiation,
        mut args: Vec<Vec<u8>>,
    ) -> Result<Vec<Vec<u8>>, PartialVMError>
    where
        S: MoveOSResolver,
    {
        func.parameters.iter().enumerate().for_each(|(i, t)| {
            if is_signer(t) {
                let signer = MoveValue::Signer(self.tx_context.sender());
                args.insert(
                    i,
                    signer
                        .simple_serialize()
                        .expect("serialize signer should success"),
                );
            } 

            if as_struct(session, t)
                .map(|t| is_storage_context(&t))
                .unwrap_or(false)
            {
                args.insert(i, self.to_bytes());
            }
        });
        Ok(args)
    }
}

fn is_signer(t: &Type) -> bool {
    matches!(t, Type::Signer) || matches!(t, Type::Reference(r) if matches!(**r, Type::Signer))
}

fn as_struct<T>(session: &Session<T>, t: &Type) -> Option<Arc<StructType>>
where
    T: MoveOSResolver,
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

pub fn as_struct_no_panic<T>(session: &Session<T>, t: &Type) -> Option<Arc<StructType>>
where
    T: MoveOSResolver,
{
    match t {
        Type::Struct(s) | Type::StructInstantiation(s, _) => session.get_struct_type(*s),
        Type::Reference(r) => as_struct_no_panic(session, r),
        Type::MutableReference(r) => as_struct_no_panic(session, r),
        _ => None,
    }
}

pub fn is_storage_context(t: &StructType) -> bool {
    t.module.address() == &StorageContext::ADDRESS
        && t.module.name() == StorageContext::module_identifier().as_ident_str()
        && t.name == StorageContext::struct_identifier()
}
