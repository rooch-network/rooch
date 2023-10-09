// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::raw_table::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "table_extension", [
    [.common.load_base, "common.load_base", (5 + 1) * MUL],
    [.common.load_per_byte, "common.load_per_byte", (5 + 1) * MUL],
    [.common.load_failure, "common.load_failure", (5 + 1) * MUL],
    [.add_box.base, "add_box.base", (5 + 1) * MUL],
    [.add_box.per_byte_serialized, "add_box.per_byte_serialized", (5 + 1) * MUL],
    [.borrow_box.base, "borrow_box.base", (5 + 1) * MUL],
    [.borrow_box.per_byte_serialized, "borrow_box.per_byte_serialized", (5 + 1) * MUL],
    [.contains_box.base, "contains_box.base", (5 + 1) * MUL],
    [.contains_box.per_byte_serialized, "contains_box.per_byte_serialized", (5 + 1) * MUL],
    [.remove_box.base, "remove_box.base", (5 + 1) * MUL],
    [.remove_box.per_byte_serialized, "remove_box.per_byte_serialized", (5 + 1) * MUL],
    [.drop_unchecked_box.base, "drop_unchecked_box.base", (5 + 1) * MUL],
    [.box_length.base, "box_length.base", (5 + 1) * MUL],
]);
