// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::genesis {

    use std::option;
    use moveos_std::tx_context;
    use rooch_framework::account;
    use rooch_framework::auth_validator_registry;
    use rooch_framework::builtin_validators;
    use rooch_framework::chain_id;
    use rooch_framework::coin;
    use rooch_framework::account_coin_store;
    use rooch_framework::gas_coin;
    use rooch_framework::transaction_fee;
    use rooch_framework::address_mapping;
    use rooch_framework::onchain_config;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};


    const ErrorGenesisInit: u64 = 1;

    /// GenesisContext is a genesis init parameters in the TxContext.
    struct GenesisContext has copy,store,drop{
        chain_id: u64,
        /// Sequencer account
        sequencer: BitcoinAddress, 
    }

    fun init(){
        let genesis_account = &account::create_system_account(@rooch_framework);
        let genesis_context_option = tx_context::get_attribute<GenesisContext>();
        assert!(option::is_some(&genesis_context_option), ErrorGenesisInit);
        let genesis_context = option::extract(&mut genesis_context_option);
        chain_id::genesis_init(genesis_account, genesis_context.chain_id);
        auth_validator_registry::genesis_init(genesis_account);
        builtin_validators::genesis_init(genesis_account);
        coin::genesis_init(genesis_account);
        account_coin_store::genesis_init(genesis_account);
        gas_coin::genesis_init(genesis_account);
        transaction_fee::genesis_init(genesis_account);
        address_mapping::genesis_init(genesis_account);
        let sequencer_addr = bitcoin_address::to_rooch_address(&genesis_context.sequencer);
        onchain_config::genesis_init(genesis_account, sequencer_addr);

        // Some test cases use framework account as sequencer, it may already exist
        if(!moveos_std::account::exists_at(sequencer_addr)){
            account::create_account(sequencer_addr);
            address_mapping::bind_bitcoin_address(sequencer_addr, genesis_context.sequencer);
        };
        // give some gas coin to the sequencer
        gas_coin::faucet(sequencer_addr, 1000000_00000000u256);
    }


    #[test_only]
    use moveos_std::genesis;

    #[test_only]
    /// init the genesis context for test
    public fun init_for_test(){
        let genesis_account = moveos_std::signer::module_signer<GenesisContext>();
        let sequencer = bitcoin_address::from_string(&std::string::utf8(b"1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"));
        tx_context::add_attribute_via_system(&genesis_account, GenesisContext{chain_id: 3, sequencer});
        genesis::init_for_test();
        init();
    }
}
