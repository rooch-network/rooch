// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::moveos_vm::MoveOSSession;
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::{language_storage::TypeTag, vm_status::StatusCode};
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_runtime::session::{LoadedFunctionInstantiation, Session};
use move_vm_types::loaded_data::runtime_types::{StructType, Type};
use moveos_common::types::{ClassifiedGasMeter, SwitchableGasMeter};
use moveos_object_runtime::resolved_arg::{ObjectArg, ResolvedArg};
use moveos_object_runtime::TypeLayoutLoader;
use moveos_types::state::ObjectState;
use moveos_types::{
    move_std::{ascii::MoveAsciiString, string::MoveString},
    moveos_std::object::{is_object_struct, ObjectID},
    state::MoveState,
};
use moveos_types::{
    moveos_std::object::Object,
    state::{MoveStructType, PlaceholderStruct},
    state_resolver::MoveOSResolver,
};
use std::ops::Deref;
use std::sync::Arc;
use std::vec::IntoIter;

impl<'r, 'l, S, G> MoveOSSession<'r, 'l, S, G>
where
    S: MoveOSResolver,
    G: SwitchableGasMeter + ClassifiedGasMeter,
{
    pub fn resolve_argument(
        &self,
        func: &LoadedFunctionInstantiation,
        args: Vec<Vec<u8>>,
        location: Location,
    ) -> VMResult<Vec<ResolvedArg>> {
        let mut resolved_args = Vec::with_capacity(args.len());

        let mut args = args.into_iter();
        let parameters = func.parameters.clone();

        //fill the type arguments to parameter type
        let parameters = parameters
            .into_iter()
            .map(|ty| ty.subst(&func.type_arguments))
            .collect::<PartialVMResult<Vec<_>>>()
            .map_err(|err| err.finish(location.clone()))?;

        //check object id
        for parameter in parameters.iter() {
            self.resolve_arg(parameter, &mut resolved_args, &mut args, location.clone())?;
        }

        if args.next().is_some() {
            return Err(
                PartialVMError::new(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH)
                    .with_message("argument length mismatch, too many args".to_string())
                    .finish(location.clone()),
            );
        }

        if func.parameters.len() != resolved_args.len() {
            return Err(
                PartialVMError::new(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH)
                    .with_message(format!(
                        "Invalid argument length, expect:{}, got:{}",
                        func.parameters.len(),
                        resolved_args.len()
                    ))
                    .finish(location.clone()),
            );
        }
        Ok(resolved_args)
    }

    pub fn load_arguments(&mut self, resolved_args: Vec<ResolvedArg>) -> VMResult<Vec<Vec<u8>>> {
        let mut object_runtime = self.object_runtime.write();
        object_runtime.load_arguments(self, &resolved_args)?;
        Ok(resolved_args
            .into_iter()
            .map(|arg| arg.into_serialized_arg())
            .collect())
    }

    fn load_object_and_check_type(
        &self,
        object_id: &ObjectID,
        object_type: TypeTag,
        location: Location,
    ) -> VMResult<ObjectState> {
        let object = self
            .remote
            .get_object(object_id)
            .map_err(|e| {
                PartialVMError::new(StatusCode::STORAGE_ERROR)
                    .with_message(format!("Failed to resolve object state: {:?}", e))
                    .finish(location.clone())
            })?
            .ok_or_else(|| {
                PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                    .with_message(format!("Object not found: {:?}", object_id))
                    .finish(location.clone())
            })?;

        if !object.match_type(&object_type) {
            return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                .with_message(format!(
                    "Invalid object type, object type in argument:{:?}, object type in store:{:?}",
                    object_type,
                    object.object_type()
                ))
                .finish(location.clone()));
        }
        if object.is_dynamic_field() {
            return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                .with_message("Dynamic field object can not as argument".to_string())
                .finish(location.clone()));
        }

        Ok(object)
    }

    fn resolve_arg(
        &self,
        parameter: &Type,
        resolved_args: &mut Vec<ResolvedArg>,
        args: &mut IntoIter<Vec<u8>>,
        location: Location,
    ) -> VMResult<()> {
        if is_signer(parameter) {
            resolved_args.push(ResolvedArg::signer(self.tx_context().sender()));
        } else if let Type::Vector(v) = parameter {
            let type_ = v.deref();
            let mut object_vec = vec![];

            if let Some(struct_arg_type) = as_struct_no_panic(&self.session, type_) {
                if is_object(&struct_arg_type) {
                    let vector_arg = args.next().ok_or_else(|| {
                        PartialVMError::new(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH)
                            .with_message("Argument length mismatch".to_string())
                            .finish(location.clone())
                    })?;

                    let object_id_list = bcs::from_bytes::<Vec<ObjectID>>(&vector_arg)
                        .expect("MoveValue::simple_deserialize failed");

                    let object_type_tag = self.get_type_tag_option(type_).ok_or_else(|| {
                        PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                            .with_message("Resolve parameter type failed".to_string())
                            .finish(location.clone())
                    })?;

                    //The Object<T>'s T type
                    let object_type = get_object_type(&object_type_tag).ok_or_else(|| {
                        PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                            .with_message("Resolve object type failed".to_string())
                            .finish(location.clone())
                    })?;

                    // read all the object from DB
                    for object_id in object_id_list.iter() {
                        let object = self.load_object_and_check_type(
                            object_id,
                            object_type.clone(),
                            location.clone(),
                        )?;

                        let sender = self.tx_context().sender();
                        if !object.is_shared() && object.owner() != sender {
                            return Err(PartialVMError::new(StatusCode::NO_ACCOUNT_ROLE)
                                .with_message(format!(
                                    "Object owner mismatch, object owner:{:?}, sender:{:?}",
                                    object.owner(),
                                    sender
                                ))
                                .finish(location.clone()));
                        }

                        object_vec.push(ObjectArg::Value(object.clone()));
                    }

                    if !object_vec.is_empty() {
                        resolved_args.push(ResolvedArg::ObjectVector(object_vec));
                    }
                } else {
                    self.resolve_arg(type_, resolved_args, args, location.clone())?;
                }
            } else {
                self.resolve_arg(type_, resolved_args, args, location.clone())?;
            }
        } else if let Some(struct_arg_type) = as_struct_no_panic(&self.session, parameter) {
            if is_object(&struct_arg_type) {
                let object_type_tag = self.get_type_tag_option(parameter).ok_or_else(|| {
                    PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                        .with_message("Resolve parameter type failed".to_string())
                        .finish(location.clone())
                })?;
                //The Object<T>'s T type
                let object_type = get_object_type(&object_type_tag).ok_or_else(|| {
                    PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                        .with_message("Resolve object type failed".to_string())
                        .finish(location.clone())
                })?;
                let arg = args.next().ok_or_else(|| {
                    PartialVMError::new(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH)
                        .with_message("Argument length mismatch".to_string())
                        .finish(location.clone())
                })?;
                let object_id = ObjectID::from_bytes(arg).map_err(|e| {
                    PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                        .with_message(format!("Invalid object id: {:?}", e))
                        .finish(location.clone())
                })?;
                //TODO we can directly resolve args via ObjectRuntime, and remove the load_arguments functions.
                let object = self.load_object_and_check_type(
                    &object_id,
                    object_type.clone(),
                    location.clone(),
                )?;
                match parameter {
                    Type::Reference(_r) => {
                        // Any one can pass any &Object<T>
                        resolved_args.push(ResolvedArg::object_by_ref(object));
                    }
                    Type::MutableReference(_r) => {
                        // If the object is shared, the object can be passed by mutref
                        // If the object is not shared, the object can be passed by mutref only if the sender is the owner
                        if object.is_frozen() {
                            return Err(PartialVMError::new(StatusCode::NO_ACCOUNT_ROLE)
                                .with_message(format!(
                                    "Object is frozen, object id:{:?}",
                                    object_id
                                ))
                                .finish(location.clone()));
                        }
                        let sender = self.tx_context().sender();
                        if !object.is_shared() && object.owner() != sender {
                            return Err(PartialVMError::new(StatusCode::NO_ACCOUNT_ROLE)
                                .with_message(format!(
                                    "Object owner mismatch, object owner:{:?}, sender:{:?}",
                                    object.owner(),
                                    sender
                                ))
                                .finish(location.clone()));
                        }
                        resolved_args.push(ResolvedArg::object_by_mutref(object));
                    }
                    Type::StructInstantiation(_, _) => {
                        // Only the owner can pass `Object<T>` by value
                        if object.is_frozen() {
                            return Err(PartialVMError::new(StatusCode::NO_ACCOUNT_ROLE)
                                .with_message(format!(
                                    "Object is frozen, object id:{:?}",
                                    object_id
                                ))
                                .finish(location.clone()));
                        }
                        let sender = self.tx_context().sender();
                        if object.owner() != sender {
                            return Err(PartialVMError::new(StatusCode::NO_ACCOUNT_ROLE)
                                .with_message(format!(
                                    "Object owner mismatch, object owner:{:?}, sender:{:?}",
                                    object.owner(),
                                    sender
                                ))
                                .finish(location.clone()));
                        }
                        resolved_args.push(ResolvedArg::object_by_value(object));
                    }
                    _ => {
                        return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                            .with_message(
                                "Object type only support `&Object<T>`, `&mut Object<T>`, and `Object<T>`".to_string())
                            .finish(location.clone()));
                    }
                }
            } else {
                //Other pure value Struct args
                //If the session is read_only, only allow any pure value struct, otherwise, only allow the allowed struct
                if self.read_only || is_allowed_argument_struct(&struct_arg_type) {
                    let arg = args.next().ok_or_else(|| {
                        PartialVMError::new(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH)
                            .with_message("Argument length mismatch".to_string())
                            .finish(location.clone())
                    })?;
                    resolved_args.push(ResolvedArg::pure(arg));
                } else {
                    return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                        .with_message(format!("Unsupported arg type {:?}", struct_arg_type))
                        .finish(location.clone()));
                }
            }
        } else {
            //Other non-struct pure value args
            let arg = args.next().ok_or_else(|| {
                PartialVMError::new(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH)
                    .with_message("argument length mismatch".to_string())
                    .finish(location.clone())
            })?;
            resolved_args.push(ResolvedArg::pure(arg));
        }

        Ok(())
    }
}

impl<'r, 'l, S, G> TypeLayoutLoader for MoveOSSession<'r, 'l, S, G>
where
    S: MoveOSResolver,
    G: SwitchableGasMeter + ClassifiedGasMeter,
{
    fn get_type_layout(
        &self,
        type_tag: &TypeTag,
    ) -> move_binary_format::errors::PartialVMResult<move_core_types::value::MoveTypeLayout> {
        self.session
            .get_type_layout(type_tag)
            .map_err(|e| e.to_partial())
    }

    fn type_to_type_layout(
        &self,
        ty: &Type,
    ) -> move_binary_format::errors::PartialVMResult<move_core_types::value::MoveTypeLayout> {
        let type_tag = self.type_to_type_tag(ty)?;
        self.get_type_layout(&type_tag)
    }

    fn type_to_type_tag(&self, ty: &Type) -> move_binary_format::errors::PartialVMResult<TypeTag> {
        self.session.get_type_tag(ty).map_err(|e| e.to_partial())
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

pub(crate) fn is_object(t: &StructType) -> bool {
    t.module.address() == &Object::<PlaceholderStruct>::ADDRESS
        && t.module.name() == Object::<PlaceholderStruct>::module_identifier().as_ident_str()
        && t.name == Object::<PlaceholderStruct>::struct_identifier()
}

pub fn get_object_type(type_tag: &TypeTag) -> Option<TypeTag> {
    match type_tag {
        TypeTag::Struct(s) => {
            if is_object_struct(s) {
                s.type_params.first().cloned()
            } else {
                None
            }
        }
        _ => None,
    }
}

// Keep consistent with verifier is_allowed_input_struct
fn is_allowed_argument_struct(t: &StructType) -> bool {
    (t.module.address() == &MoveString::ADDRESS
        && t.module.name() == MoveString::module_identifier().as_ident_str()
        && t.name == MoveString::struct_identifier())
        || (t.module.address() == &MoveAsciiString::ADDRESS
            && t.module.name() == MoveAsciiString::module_identifier().as_ident_str()
            && t.name == MoveAsciiString::struct_identifier())
        || (t.module.address() == &ObjectID::ADDRESS
            && t.module.name() == ObjectID::module_identifier().as_ident_str()
            && t.name == ObjectID::struct_identifier())
}
