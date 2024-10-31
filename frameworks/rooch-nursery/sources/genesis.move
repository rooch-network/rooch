// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_nursery::genesis {
    use rooch_nursery::ethereum;
    use rooch_nursery::tick_info;
    use rooch_nursery::inscribe_factory;
    use rooch_nursery::ethereum_validator;
    use rooch_nursery::ton_validator;

    const ErrorInvalidChainId: u64 = 1;

    struct GenesisContext has copy,store,drop{
    }

    fun init(genesis_account: &signer){
        ethereum::genesis_init(genesis_account);
        tick_info::genesis_init();
        inscribe_factory::genesis_init();
        ethereum_validator::genesis_init();
        ton_validator::genesis_init();
    }

    #[test_only]
    /// init the genesis context for test
    public fun init_for_test(){
        rooch_framework::genesis::init_for_test();
        let genesis_account = moveos_std::signer::module_signer<GenesisContext>();
        init(&genesis_account);
    }
}