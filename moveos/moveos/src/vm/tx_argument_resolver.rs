// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::moveos_vm::MoveOSSession;
use crate::gas::SwitchableGasMeter;
use move_binary_format::errors::{Location, PartialVMError, VMResult};
use move_core_types::{
    language_storage::{StructTag, TypeTag},
    value::MoveValue,
    vm_status::StatusCode,
};
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_runtime::session::{LoadedFunctionInstantiation, Session};
use move_vm_types::loaded_data::runtime_types::{StructType, Type};
use moveos_types::moveos_std::object_id::ObjectID;
use moveos_types::{
    moveos_std::{context::Context, object::Object},
    state::{MoveStructType, PlaceholderStruct},
    state_resolver::MoveOSResolver,
};
use std::sync::Arc;

impl<'r, 'l, 'b, S, G> MoveOSSession<'r, 'l, 'b, S, G>
where
    S: MoveOSResolver,
    G: SwitchableGasMeter,
{
    pub fn resolve_argument(
        &self,
        func: &LoadedFunctionInstantiation,
        mut args: Vec<Vec<u8>>,
    ) -> VMResult<Vec<Vec<u8>>> {
        //fill in signer and context
        func.parameters.iter().enumerate().for_each(|(i, t)| {
            if is_signer(t) {
                let signer = MoveValue::Signer(self.ctx.tx_context.sender());
                args.insert(
                    i,
                    signer
                        .simple_serialize()
                        .expect("serialize signer should success"),
                );
            }
            let struct_opt = as_struct_no_panic(&self.session, t);
            if struct_opt.as_ref().map(|t| is_context(t)).unwrap_or(false) {
                args.insert(i, self.ctx.to_bytes());
            }
        });

        //check object id
        if func.parameters.len() != args.len() {
            return Err(
                PartialVMError::new(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH)
                    .with_message(format!(
                        "Invalid argument length, expect:{}, got:{}",
                        func.parameters.len(),
                        args.len()
                    ))
                    .finish(Location::Undefined),
            );
        }
        for (paramter, arg) in func.parameters.iter().zip(args.iter()) {
            let type_tag_opt = get_type_tag(&self.session, paramter)?;
            if let Some(t) = type_tag_opt {
                if let Some(object_type) = get_object_type(&t) {
                    let object_id = ObjectID::from_bytes(arg).map_err(|e| {
                        PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                            .with_message(format!("Invalid object id: {:?}", e))
                            .finish(Location::Undefined)
                    })?;
                    let state = self
                        .remote
                        .resolve_object_state(&object_id)
                        .map_err(|e| {
                            PartialVMError::new(StatusCode::STORAGE_ERROR)
                                .with_message(format!("Failed to resolve object state: {:?}", e))
                                .finish(Location::Undefined)
                        })?
                        .ok_or_else(|| {
                            PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                                .with_message(format!("Object not found: {:?}", object_id))
                                .finish(Location::Undefined)
                        })?;
                    let object = state.as_raw_object().map_err(|e| {
                        PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                            .with_message(format!("Invalid object state: {:?}", e))
                            .finish(Location::Undefined)
                    })?;
                    if let TypeTag::Struct(s) = object_type {
                        if s.as_ref() != &object.value.struct_tag {
                            return Err(PartialVMError::new(
                                StatusCode::TYPE_MISMATCH,
                            )
                            .with_message(format!(
                                "Invalid object type, object type in argument:{:?}, object type in store:{:?}",
                                s, object.value.struct_tag
                            )).finish(Location::Undefined));
                        }
                    } else {
                        return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                            .with_message(format!(
                                "Object type should be struct, got:{:?}",
                                object_type
                            ))
                            .finish(Location::Undefined));
                    }
                    match paramter {
                        Type::Reference(_r) => {
                            // Any one can get any &Object<T>
                        }
                        Type::MutableReference(_r) => {
                            // Only the owner can get &mut Object<T>
                            if object.is_frozen() {
                                return Err(PartialVMError::new(StatusCode::NO_ACCOUNT_ROLE)
                                    .with_message(format!(
                                        "Object is frozen, object id:{:?}",
                                        object_id
                                    ))
                                    .finish(Location::Undefined));
                            }
                            if !object.is_shared() && object.owner != self.ctx.tx_context.sender() {
                                return Err(PartialVMError::new(StatusCode::NO_ACCOUNT_ROLE)
                                    .with_message(format!(
                                        "Object owner mismatch, object owner:{:?}, sender:{:?}",
                                        object.owner,
                                        self.ctx.tx_context.sender()
                                    ))
                                    .finish(Location::Undefined));
                            }
                        }
                        _ => {
                            return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                                .with_message(
                                    "Object type only support `&Object<T>` and `&mut Object<T>`, do not support `Object<T>`".to_string())
                                .finish(Location::Undefined));
                        }
                    }
                }
            }
        }
        Ok(args)
    }

    pub fn load_argument(&mut self, _func: &LoadedFunctionInstantiation, _args: &[Vec<u8>]) {
        //TODO load the object argument to the session
        // We need to refactor the raw table, migrate the TableData to StorageContext.
    }
}

fn is_signer(t: &Type) -> bool {
    matches!(t, Type::Signer) || matches!(t, Type::Reference(r) if matches!(**r, Type::Signer))
}

pub fn as_struct_no_panic<T>(session: &Session<T>, t: &Type) -> Option<Arc<StructType>>
where
    T: TransactionCache,
{
    match t {
        Type::Struct(s) | Type::StructInstantiation(s, _) => session.get_struct_type(*s),
        Type::Reference(r) => as_struct_no_panic(session, r),
        Type::MutableReference(r) => as_struct_no_panic(session, r),
        _ => None,
    }
}

pub fn get_type_tag<T>(session: &Session<T>, t: &Type) -> VMResult<Option<TypeTag>>
where
    T: TransactionCache,
{
    match t {
        Type::Struct(_) | Type::StructInstantiation(_, _) => Ok(Some(session.get_type_tag(t)?)),
        Type::Reference(r) => get_type_tag(session, r),
        Type::MutableReference(r) => get_type_tag(session, r),
        _ => Ok(None),
    }
}

pub(crate) fn is_context(t: &StructType) -> bool {
    t.module.address() == &Context::ADDRESS
        && t.module.name() == Context::module_identifier().as_ident_str()
        && t.name == Context::struct_identifier()
}

fn is_object_struct(t: &StructTag) -> bool {
    Object::<PlaceholderStruct>::struct_tag_match_without_type_param(t)
}

pub fn get_object_type(type_tag: &TypeTag) -> Option<TypeTag> {
    match type_tag {
        TypeTag::Struct(s) => {
            if is_object_struct(s) {
                s.type_params.get(0).cloned()
            } else {
                None
            }
        }
        _ => None,
    }
}
