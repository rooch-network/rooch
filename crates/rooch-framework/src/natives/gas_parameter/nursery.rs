// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use move_stdlib::natives::NurseryGasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(NurseryGasParameters, "nursery", [
    [.event.write_to_event_store.unit_cost, "event.write_to_event_store.unit_cost", (5 + 1) * MUL],
    [.debug.print.base_cost, optional "debug.print.base_cost", MUL],
    [.debug.print_stack_trace.base_cost, optional "debug.print_stack_trace.base_cost",  MUL],
]);
