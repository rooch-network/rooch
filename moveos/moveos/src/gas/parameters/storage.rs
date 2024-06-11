// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::gas::gas_member::MUL;
use crate::gas::table::StorageGasParameter;

crate::gas::native::define_gas_parameters_for_natives!(StorageGasParameter, "storage", [
    [.io_read_price, "io_read_price", (5 + 1) * MUL],
    [.storage_fee_per_event_byte, "storage_fee_per_event_byte", (5 + 1) * MUL],
    [.storage_fee_per_op_new_byte, "storage_fee_per_op_new_byte", (5 + 1) * MUL],
    [.storage_fee_per_op_modify_byte, "storage_fee_per_op_modify_byte", (5 + 1) * MUL],
    [.storage_fee_per_op_delete, "storage_fee_per_op_delete", (5 + 1) * MUL],
    [.storage_fee_per_transaction_byte.tx_size_leve_1, "storage_fee_tx_size_leve_1", 1024],
    [.storage_fee_per_transaction_byte.tx_size_leve_2, "storage_fee_tx_size_leve_2", 102400],
    [.storage_fee_per_transaction_byte.tx_size_leve_3, "storage_fee_tx_size_leve_3", 1024000],
    [.storage_fee_per_transaction_byte.tx_size_leve_4, "storage_fee_tx_size_leve_4", 10240000],
    [.storage_fee_per_transaction_byte.tx_size_gas_parameter_level_1, "storage_fee_tx_size_gas_parameter_level_1", 1],
    [.storage_fee_per_transaction_byte.tx_size_gas_parameter_level_2, "storage_fee_tx_size_gas_parameter_level_2", 2],
    [.storage_fee_per_transaction_byte.tx_size_gas_parameter_level_3, "storage_fee_tx_size_gas_parameter_level_3", 4],
    [.storage_fee_per_transaction_byte.tx_size_gas_parameter_level_4, "storage_fee_tx_size_gas_parameter_level_4", 16],
]);
