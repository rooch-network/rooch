// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// Copyright © Aptos Foundation
// SPDX-License-Identifier: Apache-2.0

// Source code from https://github.com/aptos-labs/aptos-core/blob/c76c6b0fc3a1b8e21b6ba2f77151ca20ea31ca32/aptos-move/moveos_stdlib/src/natives/type_info.rs#L1
// TODO use the SafeNativeContext

use move_binary_format::errors::PartialVMResult;
use move_core_types::{
    gas_algebra::{InternalGas, InternalGasPerByte, NumBytes},
    language_storage::{StructTag, TypeTag},
};
use move_vm_runtime::native_functions::{NativeContext, NativeFunction};
use move_vm_types::{
    loaded_data::runtime_types::Type,
    natives::function::NativeResult,
    values::{Struct, Value},
};
use smallvec::{smallvec, SmallVec};
use std::{collections::VecDeque, fmt::Write, sync::Arc};

const E_TYPE_MISMATCH: u64 = 1;

fn type_of_internal(struct_tag: &StructTag) -> Result<SmallVec<[Value; 1]>, std::fmt::Error> {
    let mut name = struct_tag.name.to_string();
    if let Some(first_ty) = struct_tag.type_params.first() {
        write!(name, "<")?;
        write!(name, "{}", first_ty)?;
        for ty in struct_tag.type_params.iter().skip(1) {
            write!(name, ", {}", ty)?;
        }
        write!(name, ">")?;
    }

    let struct_value = Struct::pack(vec![
        Value::address(struct_tag.address),
        Value::vector_u8(struct_tag.module.as_bytes().to_vec()),
        Value::vector_u8(name.as_bytes().to_vec()),
    ]);
    Ok(smallvec![Value::struct_(struct_value)])
}

/***************************************************************************************************
 * native fun type_of
 *
 *   Returns the structs Module Address, Module Name and the Structs Name.
 *
 *   gas cost: base_cost + unit_cost * type_size
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct TypeOfGasParameters {
    pub base: InternalGas,
    pub per_byte_in_str: InternalGasPerByte,
}

fn native_type_of(
    gas_params: &TypeOfGasParameters,
    context: &mut NativeContext,
    ty_args: Vec<Type>,
    arguments: VecDeque<Value>,
) -> PartialVMResult<NativeResult> {
    debug_assert!(ty_args.len() == 1);
    debug_assert!(arguments.is_empty());

    let type_tag = context.type_to_type_tag(&ty_args[0])?;

    let cost = gas_params.base
        + if gas_params.per_byte_in_str > 0.into() {
            let type_tag_str = type_tag.to_string();
            gas_params.per_byte_in_str * NumBytes::new(type_tag_str.len() as u64)
        } else {
            0.into()
        };

    if let TypeTag::Struct(struct_tag) = type_tag {
        Ok(NativeResult::ok(
            cost,
            type_of_internal(&struct_tag).expect("type_of should never fail."),
        ))
    } else {
        Ok(NativeResult::err(cost, E_TYPE_MISMATCH))
    }
}

//TODO implement a generic make_native function to replace all make_native_xxx function
pub fn make_native_type_of(gas_params: TypeOfGasParameters) -> NativeFunction {
    Arc::new(move |context, ty_args, args| native_type_of(&gas_params, context, ty_args, args))
}

/***************************************************************************************************
 * module
 *
 **************************************************************************************************/
#[derive(Debug, Clone)]
pub struct GasParameters {
    pub type_of: TypeOfGasParameters,
}
impl GasParameters {
    pub fn zeros() -> GasParameters {
        Self {
            type_of: TypeOfGasParameters {
                base: 0.into(),
                per_byte_in_str: 0.into(),
            },
        }
    }
}

pub fn make_all(gas_params: GasParameters) -> impl Iterator<Item = (String, NativeFunction)> {
    let natives = [("type_of", make_native_type_of(gas_params.type_of))];

    crate::natives::helpers::make_module_natives(natives)
}

#[cfg(test)]
mod tests {
    use super::*;
    use move_core_types::{account_address::AccountAddress, identifier::Identifier};
    use move_vm_types::values::VMValueCast;

    #[test]
    fn test_type_of_internal() {
        let dummy_st = StructTag {
            address: AccountAddress::random(),
            module: Identifier::new("DummyModule").unwrap(),
            name: Identifier::new("DummyStruct").unwrap(),
            type_params: vec![TypeTag::Vector(Box::new(TypeTag::U8))],
        };

        let dummy_as_strings = dummy_st.to_string();
        let mut dummy_as_strings = dummy_as_strings.split("::");
        let dummy_as_type_of = type_of_internal(&dummy_st).unwrap().pop().unwrap();
        let dummy_as_type_of: Struct = dummy_as_type_of.cast().unwrap();
        let mut dummy_as_type_of = dummy_as_type_of.unpack().unwrap();

        let account_addr =
            AccountAddress::from_hex_literal(dummy_as_strings.next().unwrap()).unwrap();
        assert!(Value::address(account_addr)
            .equals(&dummy_as_type_of.next().unwrap())
            .unwrap());
        let module = dummy_as_strings.next().unwrap().as_bytes().to_owned();
        assert!(Value::vector_u8(module)
            .equals(&dummy_as_type_of.next().unwrap())
            .unwrap());
        let name = dummy_as_strings.next().unwrap().as_bytes().to_owned();
        assert!(Value::vector_u8(name)
            .equals(&dummy_as_type_of.next().unwrap())
            .unwrap());
    }
}
