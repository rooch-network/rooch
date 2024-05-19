// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::genesis{
    use std::option;
    use moveos_std::tx_context;
    use moveos_std::signer;
    use bitcoin_move::bitcoin;
    use bitcoin_move::ord;
    use bitcoin_move::utxo;
    use bitcoin_move::network;

    const ErrorGenesisInit: u64 = 1;

    /// BitcoinGenesisContext is a genesis init config in the TxContext.
    struct BitcoinGenesisContext has copy,store,drop{
        network: u8,
        genesis_block_height: u64, 
    }

    fun init(){
        let genesis_account = signer::module_signer<BitcoinGenesisContext>();
        let genesis_context_option = tx_context::get_attribute<BitcoinGenesisContext>();
        assert!(option::is_some(&genesis_context_option), ErrorGenesisInit);
        let genesis_context = option::destroy_some(genesis_context_option);
        network::genesis_init(genesis_context.network);
        utxo::genesis_init();
        ord::genesis_init(&genesis_account);
        bitcoin::genesis_init(&genesis_account, genesis_context.genesis_block_height);
    }

    #[test_only]
    /// init the genesis context for test
    public fun init_for_test(){
        let genesis_account = moveos_std::signer::module_signer<BitcoinGenesisContext>();
        tx_context::add_attribute_via_system(&genesis_account, BitcoinGenesisContext{network: network::network_signet(), genesis_block_height: 0});
        init();
    }

    #[test]
    fun test_init(){
        init_for_test();
        let network = network::network();
        assert!(network == network::network_signet(), 1000);
    }
}