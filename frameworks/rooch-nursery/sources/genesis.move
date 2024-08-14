// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::genesis {
    use rooch_framework::chain_id;
    use rooch_nursery::ethereum;
    use rooch_nursery::tick_info;
    use rooch_nursery::bitcoin_multisign_validator;

    const ErrorInvalidChainId: u64 = 1;

    fun init(genesis_account: &signer){
        // Ensure the nursery is not running on test or main chain.
        // nursery can running on a local or dev chain or custom chain
        assert!(!chain_id::is_test() && !chain_id::is_main(), ErrorInvalidChainId);
        ethereum::genesis_init(genesis_account);
        bitcoin_multisign_validator::genesis_init();
        tick_info::genesis_init();
    }
}