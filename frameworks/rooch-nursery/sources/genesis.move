// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::genesis {
    use rooch_framework::chain_id;
    use rooch_nursery::ethereum;

    const ErrorInvalidChainId: u64 = 1;

    fun init(genesis_account: &signer){
        // Ensure the nursery is running on a local or dev chain.
        assert!(chain_id::is_local() || chain_id::is_dev(), ErrorInvalidChainId);
        ethereum::genesis_init(genesis_account);
    }
}