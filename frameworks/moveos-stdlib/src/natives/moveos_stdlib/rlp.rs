// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use log::info;
use move_binary_format::errors::{PartialVMError, PartialVMResult};
use move_core_types::account_address::AccountAddress;
use move_core_types::gas_algebra::{InternalGas, InternalGasPerByte, NumBytes};
use move_core_types::u256::{self, U256_NUM_BYTES};
use move_core_types::value::MoveTypeLayout;
use move_core_types::value::MoveValue;
use move_core_types::vm_status::StatusCode;
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    pop_arg,
    values::{values_impl::Reference, Struct, Value},
};
use primitive_types::U256 as PrimitiveU256;
use rlp::{self, Rlp, RlpStream};
use smallvec::smallvec;
use std::{collections::VecDeque, sync::Arc};

use crate::natives::helpers::make_module_natives;

const E_RLP_SERIALIZATION_FAILURE: u64 = 1;
const E_RLP_DESERIALIZATION_FAILURE: u64 = 2;

struct MoveValueWrapper {
    layout: MoveTypeLayout,
    val: MoveValue,
}

impl rlp::Encodable for MoveValueWrapper {
    fn rlp_append(&self, s: &mut RlpStream) {
        use MoveTypeLayout as L;
        match (&self.layout, &self.val) {
            (L::Struct(layout), MoveValue::Struct(struct_)) => {
                let layout_fields = layout.fields();
                let value_fields = struct_.fields();
                s.begin_list(layout_fields.len());
                for (layout, value) in layout_fields.iter().zip(value_fields) {
                    s.append(&MoveValueWrapper {
                        layout: layout.clone(),
                        val: value.clone(),
                    });
                }
            }
            (L::Bool, MoveValue::Bool(b)) => b.rlp_append(s),
            (L::U8, MoveValue::U8(i)) => i.rlp_append(s),
            (L::U16, MoveValue::U16(i)) => i.rlp_append(s),
            (L::U32, MoveValue::U32(i)) => i.rlp_append(s),
            (L::U64, MoveValue::U64(i)) => i.rlp_append(s),
            (L::U128, MoveValue::U128(i)) => i.rlp_append(s),
            (L::U256, MoveValue::U256(i)) => {
                let slice = i.to_le_bytes();
                let value = PrimitiveU256::from_little_endian(&slice);
                let leading_empty_bytes = value.leading_zeros() as usize / 8;
                let mut buffer = [0u8; U256_NUM_BYTES];
                value.to_big_endian(&mut buffer);
                s.encoder().encode_value(&buffer[leading_empty_bytes..]);
            }
            (L::Address, MoveValue::Address(a)) => a.to_vec().rlp_append(s),
            (L::Signer, MoveValue::Signer(a)) => a.to_vec().rlp_append(s),
            (L::Vector(layout), MoveValue::Vector(v)) => {
                let layout = &**layout;
                match layout {
                    L::U8 => {
                        let mut bytes = Vec::with_capacity(v.len());

                        for byte in v {
                            match byte {
                                MoveValue::U8(u8) => {
                                    bytes.push(*u8);
                                }
                                _ => unreachable!("This should not happen."),
                            }
                        }
                        bytes.rlp_append(s);
                    }
                    _ => {
                        s.begin_list(v.len());
                        for val in v {
                            s.append(&MoveValueWrapper {
                                layout: layout.clone(),
                                val: val.clone(),
                            });
                        }
                    }
                }
            }
            _ => todo!(),
        }
    }
}

fn decode_rlp(rlp: Rlp, layout: MoveTypeLayout) -> anyhow::Result<Value> {
    let value = match layout {
        MoveTypeLayout::Bool => Value::bool(rlp.as_val::<bool>()?),
        MoveTypeLayout::U8 => Value::u8(rlp.as_val::<u8>()?),
        MoveTypeLayout::U16 => Value::u16(rlp.as_val::<u16>()?),
        MoveTypeLayout::U32 => Value::u32(rlp.as_val::<u32>()?),
        MoveTypeLayout::U64 => Value::u64(rlp.as_val::<u64>()?),
        MoveTypeLayout::U128 => Value::u128(rlp.as_val::<u128>()?),
        MoveTypeLayout::U256 => {
            let bytes = rlp.as_val::<Vec<u8>>()?;
            let value = PrimitiveU256::from_big_endian(&bytes);
            let mut buffer = [0u8; U256_NUM_BYTES];
            value.to_little_endian(&mut buffer);
            Value::u256(u256::U256::from_le_bytes(&buffer))
        }
        MoveTypeLayout::Address => {
            let bytes = rlp.as_val::<Vec<u8>>()?;
            AccountAddress::try_from(bytes).map(Value::address)?
        }
        MoveTypeLayout::Signer => {
            let bytes = rlp.as_val::<Vec<u8>>()?;
            AccountAddress::try_from(bytes).map(Value::signer)?
        }
        MoveTypeLayout::Struct(ty) => {
            let mut fields = vec![];
            for (index, field_ty) in ty.into_fields().into_iter().enumerate() {
                let val = decode_rlp(rlp.at(index)?, field_ty)?;
                fields.push(val);
            }
            Value::struct_(Struct::pack(fields))
        }
        MoveTypeLayout::Vector(layout) => {
            match *layout.clone() {
                MoveTypeLayout::U8 => {
                    let bytes = rlp.as_val::<Vec<u8>>()?;
                    Value::vector_u8(bytes)
                }
                _ => {
                    let count = rlp.item_count()?;

                    let mut elements = vec![];
                    for i in 0..count {
                        let val = decode_rlp(rlp.at(i)?, *layout.clone())?;
                        elements.push(val);
                    }

                    // TODO: This API may break
                    Value::vector_for_testing_only(elements)
                }
            }
        }
    };
    Ok(value)
}

#[derive(Debug, Clone)]
pub struct ToBytesGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl ToBytesGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

/// Rust implementation of Move's `native public fun to_bytes<T>(&T): vector<u8> in rlp module`
#[inline]
fn native_to_bytes(
    gas_params: &ToBytesGasParameters,
    context: &mut NativeContext,
    mut ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(args.len() == 1);

    let mut cost = gas_params.base;

    // pop type and value
    let ref_to_val = pop_arg!(args, Reference);
    let arg_type = ty_args.pop().unwrap();

    // get type layout
    let layout = match context.type_to_type_layout(&arg_type)? {
        Some(layout) => layout,
        None => {
            return Ok(NativeResult::err(cost, E_RLP_SERIALIZATION_FAILURE));
        }
    };
    // serialize value
    let val = ref_to_val.read_ref()?.as_move_value(&layout);
    let serialized_value = rlp::encode(&MoveValueWrapper { layout, val })
        .into_iter()
        .collect::<Vec<u8>>();

    cost += gas_params.per_byte * NumBytes::new(serialized_value.len() as u64);

    Ok(NativeResult::ok(
        cost,
        smallvec![Value::vector_u8(serialized_value)],
    ))
}
pub fn make_native_to_bytes(gas_params: ToBytesGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_to_bytes(&gas_params, context, ty_args, args)
        },
    )
}

#[derive(Debug, Clone)]
pub struct FromBytesGasParameters {
    pub base: InternalGas,
    pub per_byte: InternalGasPerByte,
}

impl FromBytesGasParameters {
    pub fn zeros() -> Self {
        Self {
            base: 0.into(),
            per_byte: 0.into(),
        }
    }
}

/// Rust implementation of Move's `native public(friend) fun from_bytes<T>(vector<u8>): T in rlp module`
#[inline]
fn native_from_bytes(
    gas_params: &FromBytesGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    mut args: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert_eq!(ty_args.len(), 1);
    debug_assert_eq!(args.len(), 1);

    let mut cost = gas_params.base;

    // TODO(Gas): charge for getting the layout
    let layout = context.type_to_type_layout(&ty_args[0])?.ok_or_else(|| {
        PartialVMError::new(StatusCode::UNKNOWN_INVARIANT_VIOLATION_ERROR).with_message(format!(
            "Failed to get layout of type {:?} -- this should not happen",
            ty_args[0]
        ))
    })?;

    let bytes = pop_arg!(args, Vec<u8>);
    cost += gas_params.per_byte * NumBytes::new(bytes.len() as u64);

    // let value = Value::vector_u8(bytes.clone());
    let rlp = Rlp::new(&bytes);

    let value = match decode_rlp(rlp, layout) {
        Ok(val) => val,
        Err(err) => {
            info!("RLP deserialization error: {:?}", err);
            return Ok(NativeResult::err(cost, E_RLP_DESERIALIZATION_FAILURE));
        }
    };
    Ok(NativeResult::ok(cost, smallvec![value]))
}

pub fn make_native_from_bytes(gas_params: FromBytesGasParameters) -> NativeFunction {
    Arc::new(
        move |context, ty_args, args| -> PartialVMResult<NativeResult> {
            native_from_bytes(&gas_params, context, ty_args, args)
        },
    )
}

/***************************************************************************************************
 * module
 **************************************************************************************************/

#[derive(Debug, Clone)]
pub struct GasParameters {
    pub to_bytes: ToBytesGasParameters,
    pub from_bytes: FromBytesGasParameters,
}

impl GasParameters {
    pub fn zeros() -> Self {
        Self {
            to_bytes: ToBytesGasParameters::zeros(),
            from_bytes: FromBytesGasParameters::zeros(),
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [
        ("to_bytes", make_native_to_bytes(gas_params.to_bytes)),
        ("from_bytes", make_native_from_bytes(gas_params.from_bytes)),
    ];

    make_module_natives(natives)
}
