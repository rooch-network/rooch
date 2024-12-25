// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use super::moveos_vm::MoveOSSession;
use anyhow::bail;
use move_binary_format::errors::{Location, PartialVMError, PartialVMResult, VMResult};
use move_core_types::value::MoveValue;
use move_core_types::vm_status::VMStatus;
use move_core_types::{language_storage::TypeTag, vm_status::StatusCode};
use move_vm_runtime::data_cache::TransactionCache;
use move_vm_runtime::session::{LoadedFunctionInstantiation, Session};
use move_vm_types::loaded_data::runtime_types::{StructType, Type};
use moveos_common::types::{ClassifiedGasMeter, SwitchableGasMeter};
use moveos_object_runtime::resolved_arg::ResolvedArg;
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
use std::io::{Cursor, Read};
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
        mut args: Vec<Vec<u8>>,
        location: Location,
        load_object: bool,
    ) -> VMResult<Vec<Vec<u8>>> {
        let parameters = func.parameters.clone();

        //fill the type arguments to parameter type
        let parameters = parameters
            .into_iter()
            .map(|ty| ty.subst(&func.type_arguments))
            .collect::<PartialVMResult<Vec<_>>>()
            .map_err(|err| err.finish(location.clone()))?;

        for ty_ in parameters.iter() {
            let send_bytes = self.resolve_signer(ty_.clone());
            if !send_bytes.is_empty() {
                args.insert(0, send_bytes);
            }
        }

        let mut args = args.into_iter();

        let serialized_args =
            self.resolve_args(parameters.clone(), &mut args, load_object, location.clone())?;

        if args.next().is_some() {
            return Err(
                PartialVMError::new(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH)
                    .with_message("argument length mismatch, too many args".to_string())
                    .finish(location.clone()),
            );
        }

        if func.parameters.len() != serialized_args.len() {
            return Err(
                PartialVMError::new(StatusCode::NUMBER_OF_ARGUMENTS_MISMATCH)
                    .with_message(format!(
                        "Invalid argument length, expect:{}, got:{}",
                        func.parameters.len(),
                        serialized_args.len()
                    ))
                    .finish(location.clone()),
            );
        }

        Ok(serialized_args)
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

    fn resolve_signer(&self, type_: Type) -> Vec<u8> {
        match type_ {
            Type::Signer => self.tx_context().sender.to_vec(),
            Type::Reference(inner) => self.resolve_signer(*inner),
            _ => {
                vec![]
            }
        }
    }

    fn resolve_args(
        &self,
        parameters: Vec<Type>,
        args: &mut IntoIter<Vec<u8>>,
        load_object: bool,
        location: Location,
    ) -> VMResult<Vec<Vec<u8>>> {
        let mut res_args = vec![];
        for (ty, arg) in parameters.iter().zip(args) {
            let constructed_arg =
                self.construct_arg(ty, arg.clone(), load_object, location.clone())?;
            res_args.push(constructed_arg);
        }
        Ok(res_args)
    }

    fn construct_arg(
        &self,
        ty: &Type,
        arg: Vec<u8>,
        load_object: bool,
        location: Location,
    ) -> VMResult<Vec<u8>> {
        use Type::*;
        match ty {
            Vector(..) | Struct(..) | StructInstantiation(..) => {
                let initial_cursor_len = arg.len();
                let mut cursor = Cursor::new(&arg[..]);
                let mut new_arg = vec![];
                self.recursively_construct_arg(
                    ty,
                    &mut cursor,
                    &mut new_arg,
                    initial_cursor_len,
                    load_object,
                    location,
                )?;
                Ok(new_arg)
            }
            Bool | U8 | U16 | U32 | U64 | U128 | U256 | Address => Ok(arg),
            Signer => MoveValue::Signer(self.tx_context().sender)
                .simple_serialize()
                .ok_or_else(|| {
                    PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                        .with_message("failed to deserialize signer argument".to_string())
                        .finish(Location::Undefined)
                }),
            Reference(..) | MutableReference(..) => {
                let initial_cursor_len = arg.len();
                let mut cursor = Cursor::new(&arg[..]);
                let mut new_arg = vec![];
                self.recursively_construct_arg(
                    ty,
                    &mut cursor,
                    &mut new_arg,
                    initial_cursor_len,
                    load_object,
                    location,
                )?;
                Ok(new_arg)
            }
            TyParam(_) => Err(PartialVMError::new(StatusCode::INVALID_SIGNATURE)
                .with_message("invalid type argument".to_string())
                .finish(Location::Undefined)),
        }
    }

    fn recursively_construct_arg(
        &self,
        ty: &Type,
        cursor: &mut Cursor<&[u8]>,
        arg: &mut Vec<u8>,
        initial_cursor_len: usize,
        load_object: bool,
        location: Location,
    ) -> VMResult<()> {
        use Type::*;

        if is_signer(ty) {
            return match MoveValue::Signer(self.tx_context().sender).simple_serialize() {
                Some(mut val) => {
                    arg.append(&mut val);
                    Ok(())
                }
                None => Err(
                    PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                        .with_message("failed to deserialize signer argument".to_string())
                        .finish(Location::Undefined),
                ),
            };
        }

        if let Vector(v) = ty {
            let inner_type = v.deref();

            let mut len = get_len(cursor).map_err(|_| {
                PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                    .with_message("get length from vector data failed".to_string())
                    .finish(location.clone())
            })?;

            serialize_uleb128(len, arg);
            while len > 0 {
                self.recursively_construct_arg(
                    inner_type,
                    cursor,
                    arg,
                    initial_cursor_len,
                    load_object,
                    location.clone(),
                )?;
                len -= 1;
            }
            Ok(())
        } else if let Some(struct_arg_type) = as_struct_no_panic(&self.session, ty) {
            if is_object(&struct_arg_type) {
                let mut len = get_len(cursor).map_err(|_| {
                    PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                        .with_message("get length from vector data failed".to_string())
                        .finish(location.clone())
                })?;

                let mut object_arg_bytes = vec![];
                serialize_uleb128(len, &mut object_arg_bytes);

                while len > 0 {
                    read_n_bytes(32, cursor, &mut object_arg_bytes)?;
                    len -= 1;
                }

                let object_id = ObjectID::from_bytes(object_arg_bytes.clone()).map_err(|e| {
                    PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                        .with_message(format!("Invalid object id: {:?}", e))
                        .finish(location.clone())
                })?;

                let object_type_tag = self.get_type_tag_option(ty).ok_or_else(|| {
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

                let object = self.load_object_and_check_type(
                    &object_id,
                    object_type.clone(),
                    location.clone(),
                )?;

                match ty {
                    Reference(_) => {
                        let mut v = object.id().to_bytes();
                        arg.append(&mut v);
                    }
                    MutableReference(_) => {
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
                        let mut v = object.id().to_bytes();
                        arg.append(&mut v);
                    }
                    StructInstantiation(_, instantiation_types, _) => {
                        if let Some(Struct(struct_idx, _)) = instantiation_types.first() {
                            let first_struct_type =
                                self.get_struct_type(*struct_idx).ok_or_else(|| {
                                    PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                                        .with_message("Get struct type failed".to_string())
                                        .finish(location.clone())
                                })?;
                            let struct_abilities = first_struct_type.abilities;
                            if !(struct_abilities.has_key() && struct_abilities.has_store()) {
                                return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                                        .with_message(
                                            "The type parameter T in Object<T> lacks either store or key ability."
                                                .to_string(),
                                        )
                                        .finish(location.clone()));
                            }
                        }

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
                        let mut v = object.id().to_bytes();
                        arg.append(&mut v);
                    }
                    _ => {
                        return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                            .with_message(
                                "Object type only support `&Object<T>`, `&mut Object<T>`, and `Object<T>`".to_string())
                            .finish(location.clone()));
                    }
                }

                if load_object {
                    let mut object_runtime = self.object_runtime.write();
                    object_runtime.load_object_argument(object.id(), ty, self)?;
                }
                Ok(())
            } else if self.read_only || is_allowed_argument_struct(&struct_arg_type) {
                if is_string_or_ascii_string(&struct_arg_type) {
                    let len = get_len(cursor).map_err(|_| {
                        PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                            .with_message("get length from vector data failed".to_string())
                            .finish(location.clone())
                    })?;

                    serialize_uleb128(len, arg);
                    read_n_bytes(len, cursor, arg)?;
                } else if is_object_id(&struct_arg_type) {
                    let mut len = get_len(cursor).map_err(|_| {
                        PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                            .with_message("get length from vector data failed".to_string())
                            .finish(location.clone())
                    })?;

                    let mut object_arg_bytes = vec![];
                    serialize_uleb128(len, &mut object_arg_bytes);
                    while len > 0 {
                        read_n_bytes(32, cursor, &mut object_arg_bytes)?;
                        len -= 1;
                    }
                    arg.append(&mut object_arg_bytes);
                } else {
                    read_n_bytes(initial_cursor_len, cursor, arg)?;
                }
                Ok(())
            } else {
                return Err(PartialVMError::new(StatusCode::TYPE_MISMATCH)
                    .with_message("unsupported type.....".to_string())
                    .finish(location.clone()));
            }
        } else {
            match ty {
                Bool | U8 => read_n_bytes(1, cursor, arg),
                U16 => read_n_bytes(2, cursor, arg),
                U32 => read_n_bytes(4, cursor, arg),
                U64 => read_n_bytes(8, cursor, arg),
                U128 => read_n_bytes(16, cursor, arg),
                Address | U256 => read_n_bytes(32, cursor, arg),
                _ => Err(
                    PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                        .finish(location.clone()),
                ),
            }
        }
    }
}

impl<'r, 'l, S, G> TypeLayoutLoader for MoveOSSession<'r, 'l, S, G>
where
    S: MoveOSResolver,
    G: SwitchableGasMeter + ClassifiedGasMeter,
{
    fn get_type_layout(
        &mut self,
        type_tag: &TypeTag,
    ) -> move_binary_format::errors::PartialVMResult<move_core_types::value::MoveTypeLayout> {
        self.session
            .get_type_layout(type_tag, self.remote)
            .map_err(|e| e.to_partial())
    }

    fn type_to_type_layout(
        &mut self,
        ty: &Type,
    ) -> move_binary_format::errors::PartialVMResult<move_core_types::value::MoveTypeLayout> {
        let type_tag = self.type_to_type_tag(ty)?;
        self.get_type_layout(&type_tag)
    }

    fn type_to_type_tag(&self, ty: &Type) -> move_binary_format::errors::PartialVMResult<TypeTag> {
        self.session.get_type_tag(ty, self.remote).map_err(|e| e.to_partial())
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
        Type::Struct(s, _) | Type::StructInstantiation(s, _, _) => session.fetch_struct_ty_by_idx(*s, session.module_store),
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
                s.type_args.first().cloned()
            } else {
                None
            }
        }
        _ => None,
    }
}

fn is_string_or_ascii_string(t: &StructType) -> bool {
    (t.module.address() == &MoveString::ADDRESS
        && t.module.name() == MoveString::module_identifier().as_ident_str()
        && t.name == MoveString::struct_identifier())
        || (t.module.address() == &MoveAsciiString::ADDRESS
            && t.module.name() == MoveAsciiString::module_identifier().as_ident_str()
            && t.name == MoveAsciiString::struct_identifier())
}

fn is_object_id(t: &StructType) -> bool {
    t.module.address() == &ObjectID::ADDRESS
        && t.module.name() == ObjectID::module_identifier().as_ident_str()
        && t.name == ObjectID::struct_identifier()
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

fn get_len(cursor: &mut Cursor<&[u8]>) -> Result<usize, VMStatus> {
    match read_uleb128_as_u64(cursor) {
        Err(_) => Err(VMStatus::error(StatusCode::ZERO_SIZED_STRUCT, None)),
        Ok(len) => Ok(len as usize),
    }
}

pub fn read_u32(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<u32> {
    let mut buf = [0; 4];
    cursor.read_exact(&mut buf)?;
    Ok(u32::from_le_bytes(buf))
}

pub fn read_uleb128_as_u64(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<u64> {
    let mut value: u64 = 0;
    let mut shift = 0;
    while let Ok(byte) = read_u8(cursor) {
        let cur = (byte & 0x7F) as u64;
        if (cur << shift) >> shift != cur {
            bail!("invalid ULEB128 repr for usize");
        }
        value |= cur << shift;

        if (byte & 0x80) == 0 {
            if shift > 0 && cur == 0 {
                bail!("invalid ULEB128 repr for usize");
            }
            return Ok(value);
        }

        shift += 7;
        if shift > u64::BITS {
            break;
        }
    }
    bail!("invalid ULEB128 repr for usize");
}

pub fn read_u8(cursor: &mut Cursor<&[u8]>) -> anyhow::Result<u8> {
    let mut buf = [0; 1];
    cursor.read_exact(&mut buf)?;
    Ok(buf[0])
}

fn serialize_uleb128(mut x: usize, dest: &mut Vec<u8>) {
    while x >= 128 {
        dest.push((x | 128) as u8);
        x >>= 7;
    }
    dest.push(x as u8);
}

fn read_n_bytes(n: usize, src: &mut Cursor<&[u8]>, dest: &mut Vec<u8>) -> VMResult<()> {
    let deserialization_error = |msg: &str| -> VMResult<()> {
        Err(
            PartialVMError::new(StatusCode::FAILED_TO_DESERIALIZE_ARGUMENT)
                .with_message(format!("read_n_bytes failed {:?}", msg))
                .finish(Location::Undefined),
        )
    };
    let len = dest.len();

    // Ensure we have enough capacity for resizing.
    match dest.try_reserve(len + n) {
        Ok(()) => Ok(()),
        Err(e) => deserialization_error(&format!("Couldn't read bytes: {}", e)),
    }?;
    dest.resize(len + n, 0);
    match src.read_exact(&mut dest[len..]) {
        Ok(()) => Ok(()),
        Err(e) => deserialization_error(&format!("Couldn't read bytes: {}", e)),
    }
}
