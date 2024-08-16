// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::builtin_validators{

    use rooch_framework::auth_validator_registry;
    use rooch_framework::session_validator;
    use rooch_framework::bitcoin_validator;

    friend rooch_framework::genesis;

    const ErrorGenesisInit: u64 = 1;

    const SESSION_VALIDATOR_ID: u64 = 0;
    const BITCOIN_VALIDATOR_ID: u64 = 1;
    /// Bitcoin multisign validator is defined in bitcoin_move framework.
    const BITCOIN_MULTISIGN_VALIDATOR_ID: u64 = 2;

    public(friend) fun genesis_init(_genesis_account: &signer) {
        // NATIVE_AUTH_VALIDATOR_ID: u64 = 0;
        let id = auth_validator_registry::register_internal<session_validator::SessionValidator>();
        assert!(id == session_validator::auth_validator_id(), ErrorGenesisInit);

        // BITCOIN_AUTH_VALIDATOR_ID: u64 = 1;
        let id = auth_validator_registry::register_internal<bitcoin_validator::BitcoinValidator>();
        assert!(id == bitcoin_validator::auth_validator_id(), ErrorGenesisInit);
    }

    public fun is_builtin_auth_validator(auth_validator_id: u64): bool {
        auth_validator_id == SESSION_VALIDATOR_ID || 
        auth_validator_id == BITCOIN_VALIDATOR_ID || 
        auth_validator_id == BITCOIN_MULTISIGN_VALIDATOR_ID
    }
}
