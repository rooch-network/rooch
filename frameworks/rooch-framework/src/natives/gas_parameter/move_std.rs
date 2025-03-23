// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use move_stdlib::natives::GasParameters;

// modify should with impl From<VMConfig> for GasSchedule
crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "move_stdlib", [
    [.bcs.to_bytes.per_byte_serialized, "bcs.to_bytes.per_byte_serialized", (181 + 1) * MUL],
    [.bcs.to_bytes.failure, "bcs.to_bytes.failure", (181 + 1) * MUL],
    [.bcs.to_bytes.legacy_min_output_size,  "bcs.to_bytes.legacy_min_output_size",  MUL],

    [.hash.sha2_256.per_byte,  "hash.sha2_256.per_byte", (21 + 1) * MUL],
    [.hash.sha2_256.legacy_min_input_len,  "hash.sha2_256.legacy_min_input_len", MUL],
    [.hash.sha3_256.per_byte,  "hash.sha3_256.per_byte",  (64 + 1) * MUL],
    [.hash.sha3_256.legacy_min_input_len,  "hash.sha3_256.legacy_min_input_len",  MUL],

    [.signer.borrow_address.base, "signer.borrow_address.base", (353 + 1) * MUL],

    [.string.check_utf8.per_byte, "string.check_utf8.per_byte", (4 + 1) *  MUL],
    [.string.is_char_boundary.base, "string.is_char_boundary.base", (4 + 1) * MUL],
    [.string.sub_string.per_byte, "string.sub_string.per_byte", (4 + 1) *  MUL],
    [.string.index_of.per_byte_searched, "string.index_of.per_byte_searched", (4 + 1)  * MUL],

    // TODO: It is necessary to retain gas records to maintain compatibility.
    [.vector.length, "vector.length.base", (98 + 1) * MUL],
    [.vector.empty, "vector.empty.base", (84 + 1) * MUL],
    [.vector.borrow, "vector.borrow.base", (1334 + 1) * MUL],
    [.vector.push_back, "vector.push_back.legacy_per_abstract_memory_unit", (53 + 1) * MUL],
    [.vector.pop_back, "vector.pop_back.base", (227 + 1) * MUL],
    [.vector.destroy_empty, "vector.destroy_empty.base", (572 + 1) * MUL],
    [.vector.swap, "vector.swap.base", (1436 + 1) * MUL],
], allow_unmapped = 2);
