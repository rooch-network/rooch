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
    use rooch_framework::timestamp;
    use rooch_framework::address_mapping;
    use rooch_framework::onchain_config;


    const ErrorGenesisInit: u64 = 1;

    /// GenesisContext is a genesis init parameters in the TxContext.
    struct GenesisContext has copy,store,drop{
        chain_id: u64,
        /// genesis timestamp in microseconds
        timestamp: u64,
        /// Sequencer account
        sequencer: address, 
    }

    fun init(){
        
        let genesis_account = &account::create_account_internal(@rooch_framework);
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
        timestamp::genesis_init(genesis_account, genesis_context.timestamp);
        address_mapping::genesis_init(genesis_account);
        onchain_config::genesis_init(genesis_account, genesis_context.sequencer);

        // Some test cases use framework account as sequencer, it may already exist
        if(!moveos_std::account::exists_at(genesis_context.sequencer)){
            account::create_account_internal(genesis_context.sequencer);
        };
        // give some gas coin to the sequencer
        gas_coin::faucet(genesis_context.sequencer, 1000000_00000000u256);
    }


    #[test_only]
    use moveos_std::genesis;

    #[test_only]
    /// init the genesis context for test
    public fun init_for_test(){
        let genesis_account = moveos_std::signer::module_signer<GenesisContext>();
        tx_context::add_attribute_via_system(&genesis_account, GenesisContext{chain_id: 3, timestamp: 0, sequencer: @rooch_framework});
        genesis::init_for_test();
        init();
    }
}
