// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::crypto::encoding::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "encoding", [
    [.base58.base, "base58.base", (5 + 1) * MUL],
    [.base58.per_byte, "base58.per_byte", (5 + 1) * MUL],
    [.base58check.base, "base58check.base", (5 + 1) * MUL],
    [.base58check.per_byte, "base58check.per_byte", (5 + 1) * MUL],
    [.bech32.base, "bech32.base", (5 + 1) * MUL],
    [.bech32.per_byte, "bech32.per_byte", (5 + 1) * MUL],
    [.p2pkh.base, "p2pkh.base", (5 + 1) * MUL],
    [.p2pkh.per_byte, "p2pkh.per_byte", (5 + 1) * MUL],
    [.p2sh.base, "p2sh.base", (5 + 1) * MUL],
    [.p2sh.per_byte, "p2sh.per_byte", (5 + 1) * MUL],
]);
