// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// This module implements Ethereum validator with the ECDSA recoverable signature over Secp256k1.
module rooch_framework::ethereum_validator {

    use moveos_std::signer;

    use rooch_framework::auth_validator_registry;

    friend rooch_framework::genesis;

    /// there defines auth validator id for each blockchain
    const ETHEREUM_AUTH_VALIDATOR_ID: u64 = 3;

    const ErrorGenesisInitError: u64 = 1;
    const ErrorAddressMappingRecordNotFound: u64 = 2;
    const ErrorNotImplemented: u64 = 3;

    struct EthereumValidator has store, drop {}

    public fun auth_validator_id(): u64 {
        ETHEREUM_AUTH_VALIDATOR_ID
    }

    public(friend) fun genesis_init(){
        let system = signer::module_signer<EthereumValidator>();
        let id = auth_validator_registry::register_by_system_with_id<EthereumValidator>(&system, ETHEREUM_AUTH_VALIDATOR_ID);
        assert!(id == ETHEREUM_AUTH_VALIDATOR_ID, ErrorGenesisInitError);
    }

    /// We need to redesign the Ethereum auth validator
    /// This module is just for placeholder the AUTH_VALIDATOR_ID
    public fun validate(_authenticator_payload: vector<u8>) {
        abort ErrorNotImplemented
    }
}
