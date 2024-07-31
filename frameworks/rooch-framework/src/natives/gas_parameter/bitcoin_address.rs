// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

use crate::natives::gas_parameter::native::MUL;
use crate::natives::rooch_framework::bitcoin_address::GasParameters;

crate::natives::gas_parameter::native::define_gas_parameters_for_natives!(GasParameters, "bitcoin_address", [
    [.new.base, "parse.base", 1000 * MUL],
    [.new.per_byte, "parse.per_byte", 30 * MUL],
    [.verify_with_pk.base, "verify_bitcoin_address_with_public_key.base", 1000 * MUL],
    [.verify_with_pk.per_byte, "verify_bitcoin_address_with_public_key.per_byte", 30 * MUL],
    [.derive_multisig_xonly_pubkey_from_xonly_pubkeys.base, optional "derive_multisig_xonly_pubkey_from_xonly_pubkeys.base", 1000 * MUL],
    [.derive_multisig_xonly_pubkey_from_xonly_pubkeys.per_byte, optional "derive_multisig_xonly_pubkey_from_xonly_pubkeys.per_byte", 30 * MUL],
    [.derive_bitcoin_taproot_address_from_multisig_xonly_pubkey.base, optional "derive_bitcoin_taproot_address_from_multisig_xonly_pubkey.base", 1000 * MUL],
    [.derive_bitcoin_taproot_address_from_multisig_xonly_pubkey.per_byte, optional "derive_bitcoin_taproot_address_from_multisig_xonly_pubkey.per_byte", 30 * MUL],
]);
