// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::object::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "object", [
    [.as_ref_inner.base, "as_ref_inner.base", 500 * MUL],
    [.as_mut_ref_inner.base, "as_mut_ref_inner.base", 1000 * MUL],
    [.common.load_base, "common.load_base", 1000 * MUL],
    [.common.load_per_byte, "common.load_per_byte", 10 * MUL],
    [.common.load_failure, "common.load_failure", 5 * MUL],
    [.add_box.base, "add_box.base", 500 * MUL],
    [.add_box.per_byte_serialized, "add_box.per_byte_serialized", 10 * MUL],
    [.borrow_box.base, "borrow_box.base", 500 * MUL],
    [.borrow_box.per_byte_serialized, "borrow_box.per_byte_serialized", 10 * MUL],
    [.contains_box.base, "contains_box.base", 500 * MUL],
    [.contains_box.per_byte_serialized, "contains_box.per_byte_serialized", 10 * MUL],
    [.contains_box_with_value_type.base, "contains_box_with_value_type.base", 500 * MUL],
    [.contains_box_with_value_type.per_byte_serialized, "contains_box_with_value_type.per_byte_serialized", 10 * MUL],
    [.remove_box.base, "remove_box.base", 500 * MUL],
    [.remove_box.per_byte_serialized, "remove_box.per_byte_serialized", 10 * MUL],
]);
