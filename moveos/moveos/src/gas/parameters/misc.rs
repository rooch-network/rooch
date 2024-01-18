// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::gas::gas_member::MUL;
use crate::gas::table::AbstractValueSizeGasParameter;

crate::gas::native::define_gas_parameters_for_natives!(AbstractValueSizeGasParameter, "storage", [
    [.u8, "u8", 40],
    [.u16, "u16", 40],
    [.u32, "u32", 40],
    [.u64, "u64", 40],
    [.u128, "u128", 40],
    [.u256, "u256", 40],
    [.bool, "bool", 40],
    [.address, "address", 40],
    [.struct_, "struct", 40],
    [.vector, "vector", 40],
    [.reference, "reference", 40],

    [.per_u8_packed, "per_u8_packed", 1],
    [.per_u16_packed, "per_u16_packed", 2],
    [.per_u32_packed, "per_u32_packed", 4],
    [.per_u64_packed, "per_u64_packed", 8],
    [.per_u128_packed, "per_u128_packed", 16],
    [.per_u256_packed, "per_u128_packed", 32],
    [.per_bool_packed, "per_bool_packed", 1],
    [.per_address_packed, "per_address_packed", 32],
]);