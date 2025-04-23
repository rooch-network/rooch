// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use moveos_stdlib::natives::moveos_stdlib::event::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "events", [
    [.emit.base, "emit.base", (52 + 1) * MUL],
    [.emit.per_byte_in_str, "emit.per_byte_in_str", (5 + 1) * MUL],
    // The following parameters need to be retained, but will not be actually used
    [.emit_with_handle.base, optional "decode.base", 0],
    [.emit_with_handle.per_byte_in_str, optional "decode.per_byte_in_str", 0],
    // The following parameters will be actually used
    [.emit_with_handle.base, optional "emit_with_handle.base", 0],
    [.emit_with_handle.per_byte_in_str, optional "emit_with_handle.per_byte_in_str", 0],
]);
