// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::genesis{
    use std::option;
    use bitcoin_move::data_import_config;
    use moveos_std::tx_context;
    use moveos_std::signer;
    use bitcoin_move::light_client;
    use bitcoin_move::ord;
    use bitcoin_move::brc20;
    use bitcoin_move::utxo;
    use bitcoin_move::network;

    const ErrorGenesisInit: u64 = 1;

    /// BitcoinGenesisContext is a genesis init config in the TxContext.
    struct BitcoinGenesisContext has copy,store,drop{
        network: u8,
        data_import_mode: u8,
    }

    fun init(){
        let genesis_account = signer::module_signer<BitcoinGenesisContext>();
        let genesis_context_option = tx_context::get_attribute<BitcoinGenesisContext>();
        assert!(option::is_some(&genesis_context_option), ErrorGenesisInit);
        let genesis_context = option::destroy_some(genesis_context_option);
        network::genesis_init(genesis_context.network);
        data_import_config::genesis_init(genesis_context.data_import_mode);
        utxo::genesis_init();
        brc20::genesis_init(&genesis_account);
        ord::genesis_init(&genesis_account);
        light_client::genesis_init(&genesis_account);
    }

    #[test_only]
    /// init the genesis context for test
    public fun init_for_test(){
        let genesis_account = moveos_std::signer::module_signer<BitcoinGenesisContext>();
        tx_context::add_attribute_via_system(&genesis_account, BitcoinGenesisContext{network: network::network_signet(), data_import_mode: data_import_config::data_import_mode_none()});
        init();
    }

    #[test]
    fun test_init(){
        init_for_test();
        let network = network::network();
        assert!(network == network::network_signet(), 1000);
    }
}