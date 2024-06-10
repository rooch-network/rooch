// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0
use crate::gas::table::TxGasParameter;

crate::gas::native::define_gas_parameters_for_natives!(TxGasParameter, "tx", [
    [.tx_size_leve_1, "tx_size_leve_1", 1024],
    [.tx_size_leve_2, "tx_size_leve_2", 102400],
    [.tx_size_leve_3, "tx_size_leve_3", 1024000],
    [.tx_size_leve_4, "tx_size_leve_4", 10240000],
    [.tx_size_gas_parameter_level_1, "tx_size_gas_parameter_level_1", 1],
    [.tx_size_gas_parameter_level_2, "tx_size_gas_parameter_level_2", 2],
    [.tx_size_gas_parameter_level_3, "tx_size_gas_parameter_level_3", 4],
    [.tx_size_gas_parameter_level_4, "tx_size_gas_parameter_level_4", 16],
]);
