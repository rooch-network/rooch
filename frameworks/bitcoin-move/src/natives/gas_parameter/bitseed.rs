// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::ord::bitseed::ArgsPackingGasParameters;
use rooch_framework::natives::gas_parameter::native::MUL;

rooch_framework::natives::gas_parameter::native::define_gas_parameters_for_natives!(ArgsPackingGasParameters, "bitseed", [
    [.base, "native_pack_inscribe_generate_args.base", 100 * MUL],
    [.per_byte, "native_pack_inscribe_generate_args.per_byte", 1000 * MUL],
]);
