// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::genesis {
    use rooch_framework::chain_id;

    const ErrorInvalidChainId: u64 = 1;

    fun init(){
        // Ensure the nursery is running on a local or dev chain.
        assert!(chain_id::is_local() || chain_id::is_dev(), ErrorInvalidChainId);
    }
}