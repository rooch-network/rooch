// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::bitcoin_address::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "bitcoin_address", [
    [.parse.base, "parse.base", 1000 * MUL],
    [.parse.per_byte, "parse.per_byte", 30 * MUL],
    [.verify_bitcoin_address_with_public_key.base, "verify_bitcoin_address_with_public_key.base", 1000 * MUL],
    [.verify_bitcoin_address_with_public_key.per_byte, "verify_bitcoin_address_with_public_key.per_byte", 30 * MUL],
    [.derive_multisig_xonly_pubkey_from_public_keys.base, "derive_multisig_xonly_pubkey_from_public_keys.base", 1000 * MUL],
    [.derive_multisig_xonly_pubkey_from_public_keys.per_byte, "derive_multisig_xonly_pubkey_from_public_keys.per_byte", 30 * MUL],
    [.derive_bitcoin_taproot_address_from_multisig_xonly_pubkey.base, "derive_bitcoin_taproot_address_from_multisig_xonly_pubkey.base", 1000 * MUL],
    [.derive_bitcoin_taproot_address_from_multisig_xonly_pubkey.per_byte, "derive_bitcoin_taproot_address_from_multisig_xonly_pubkey.per_byte", 30 * MUL],
]);
