// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module noop_auth_validator::noop_validator {
    use std::vector;
    
    use rooch_framework::auth_validator;
    use rooch_framework::auth_validator_registry;

    struct NoOpValidator has store {}

    fun init() {
        auth_validator_registry::register<NoOpValidator>();
    }

    /// NoOpValidator is an auth validator that does not validate anything.
    /// It is used for testing purposes, and should not be used in production.
    /// It is only failed when the authenticator_payload is empty.
    public fun validate(authenticator_payload: vector<u8>) {
        assert!(vector::length(&authenticator_payload) > 0, auth_validator::error_validate_invalid_authenticator());
    }
}
