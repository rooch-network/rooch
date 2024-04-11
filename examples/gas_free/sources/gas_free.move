// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::gas_payer {
    

    fun gas_validate_function(): bool{
        false
    }

    fun gas_charge_post_function(__gas_used: u128): bool {
        true
    }

    // TODO: Gas free functions are not supported yet.
    // #[gas_free(gas_validate=gas_validate_function, gas_charge_post=gas_charge_post_function)]
    // public entry fun play(__sender: &signer){
    // }
}
