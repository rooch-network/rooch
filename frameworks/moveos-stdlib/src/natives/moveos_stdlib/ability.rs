// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::helpers;
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::{
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    language_storage::TypeTag,
    vm_status::StatusCode,
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::values::Struct;
use move_vm_types::{
    loaded_data::runtime_types::Type, natives::function::NativeResult, values::Value,
};
use smallvec::smallvec;
use std::collections::VecDeque;
use std::str::FromStr;

#[derive(Debug, Clone)]
pub struct GetAbilitiesGasParameters {
    pub base: Option<InternalGas>,
    pub per_byte_in_str: Option<InternalGasPerByte>,
}

impl GetAbilitiesGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: Some(InternalGas::zero()),
            per_byte_in_str: Some(InternalGasPerByte::zero()),
        }
    }

    pub fn init(base: InternalGas, per_byte: InternalGasPerByte) -> Self {
        Self {
            base: Some(base),
            per_byte_in_str: Some(per_byte),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.base.is_none() || self.per_byte_in_str.is_none()
    }
}

fn native_get_abilities(
    gas_params: &GetAbilitiesGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(args.len() == 1);
    debug_assert!(ty_args.is_empty());

    // Extract the string::String argument
    let type_arg = args.pop_back().unwrap();
    let mut fields = type_arg.value_as::<Struct>()?.unpack()?;
    let val = fields.next().ok_or_else(|| {
        PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE)
            .with_message("String must have exactly one field".to_owned())
    })?;

    // Get the bytes and convert to UTF-8 string
    let bytes = val.value_as::<Vec<u8>>()?;
    let type_str = String::from_utf8(bytes.clone()).map_err(|e| {
        PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(e.to_string())
    })?;

    // Calculate gas cost
    let cost = gas_params.base.unwrap_or_else(InternalGas::zero)
        + gas_params
            .per_byte_in_str
            .unwrap_or_else(InternalGasPerByte::zero)
            * NumBytes::new(bytes.len() as u64);

    // Parse into a TypeTag
    let type_tag = TypeTag::from_str(type_str.as_str()).map_err(|e| {
        PartialVMError::new(StatusCode::TYPE_RESOLUTION_FAILURE).with_message(e.to_string())
    })?;

    // Try to get abilities through NativeContext.abilities() if available
    let abilities_byte = if let Ok(loaded_type) = context.load_type(&type_tag) {
        // Try to use context.abilities() if it exists
        if let Ok(abilities) = context.abilities(&loaded_type) {
            abilities.into_u8()
        } else {
            // Fallback to type-based determination
            match loaded_type {
                Type::Bool
                | Type::U8
                | Type::U16
                | Type::U32
                | Type::U64
                | Type::U128
                | Type::U256 => 0x7, // COPY | DROP | STORE
                Type::Address => 0x7,   // COPY | DROP | STORE
                Type::Signer => 0x2,    // DROP only
                Type::Vector(_) => 0x7, // COPY | DROP | STORE (simplified)
                Type::Reference(_) | Type::MutableReference(_) => 0x3, // COPY | DROP
                Type::Struct(_) | Type::StructInstantiation(_, _) => 0x0, // No abilities if get abilities failed
                Type::TyParam(_) => 0x0, // No abilities for type parameters at runtime
            }
        }
    } else {
        // If we can't load the type, use the TypeTag for basic determination
        match &type_tag {
            TypeTag::Bool
            | TypeTag::U8
            | TypeTag::U16
            | TypeTag::U32
            | TypeTag::U64
            | TypeTag::U128
            | TypeTag::U256 => 0x7, // COPY | DROP | STORE
            TypeTag::Address => 0x7,   // COPY | DROP | STORE
            TypeTag::Signer => 0x2,    // DROP only
            TypeTag::Vector(_) => 0x7, // COPY | DROP | STORE
            TypeTag::Struct(_) => 0x0, // No abilities if can't load
        }
    };

    // Return the u8 value representing the abilities
    Ok(NativeResult::ok(cost, smallvec![Value::u8(abilities_byte)]))
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub get_abilities: GetAbilitiesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            get_abilities: GetAbilitiesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [(
        "native_get_abilities",
        helpers::make_native(gas_params.get_abilities, native_get_abilities),
    )];

    helpers::make_module_natives(natives)
}
