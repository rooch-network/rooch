// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::gas::gas_member::MUL;
use crate::gas::table::StorageGasParameter;

crate::gas::native::define_gas_parameters_for_natives!(StorageGasParameter, "storage", [
    [.io_read_price, "io_read_price", (5 + 1) * MUL],
    [.storage_fee_per_transaction_byte, "storage_fee_per_transaction_byte", (5 + 1) * MUL],
    [.storage_fee_per_event_byte, "storage_fee_per_event_byte", (5 + 1) * MUL],
    [.storage_fee_per_op_new_byte, "storage_fee_per_op_new_byte", (5 + 1) * MUL],
    [.storage_fee_per_op_modify_byte, "storage_fee_per_op_modify_byte", (5 + 1) * MUL],
    [.storage_fee_per_op_delete, "storage_fee_per_op_delete", (5 + 1) * MUL],
]);