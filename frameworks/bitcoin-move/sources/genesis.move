// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module bitcoin_move::genesis{
    use std::option;
    use bitcoin_move::bbn;
    use moveos_std::tx_context;
    use moveos_std::signer;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};
    use bitcoin_move::bitcoin;
    use bitcoin_move::ord;
    use bitcoin_move::utxo;
    use bitcoin_move::network;
    use bitcoin_move::pending_block;
    use bitcoin_move::bitcoin_multisign_validator;
    use bitcoin_move::multisign_account;

    const ErrorGenesisInit: u64 = 1;
    const ErrorInvalidRoochDaoAddress: u64 = 2;

    /// BitcoinGenesisContext is a genesis init config in the TxContext.
    struct BitcoinGenesisContext has copy,store,drop{
        network: u8,
        genesis_block_height: u64,
        genesis_block_hash: address,
        reorg_block_count: u64, 
        rooch_dao: MultisignAccountConfig,
    }

    struct MultisignAccountConfig has copy,store,drop{
        multisign_bitcoin_address: BitcoinAddress,
        threshold: u64,
        participant_public_keys: vector<vector<u8>>,
    }

    fun init(){
        let genesis_account = signer::module_signer<BitcoinGenesisContext>();
        let genesis_context_option = tx_context::get_attribute<BitcoinGenesisContext>();
        assert!(option::is_some(&genesis_context_option), ErrorGenesisInit);
        let genesis_context = option::destroy_some(genesis_context_option);
        network::genesis_init(genesis_context.network);
        utxo::genesis_init();
        ord::genesis_init();
        bbn::genesis_init();
        bitcoin::genesis_init(&genesis_account, genesis_context.genesis_block_height, genesis_context.genesis_block_hash);
        pending_block::genesis_init(genesis_context.reorg_block_count);
        bitcoin_multisign_validator::genesis_init();
        let rooch_dao_address = multisign_account::initialize_multisig_account(genesis_context.rooch_dao.threshold, genesis_context.rooch_dao.participant_public_keys);
        assert!(rooch_dao_address == bitcoin_address::to_rooch_address(&genesis_context.rooch_dao.multisign_bitcoin_address), ErrorInvalidRoochDaoAddress);
    }

    #[test_only]
    /// init the genesis context for test
    public fun init_for_test(){
        rooch_framework::genesis::init_for_test();
        let genesis_account = moveos_std::signer::module_signer<BitcoinGenesisContext>();
        tx_context::add_attribute_via_system(&genesis_account, 
            BitcoinGenesisContext{
                network: network::network_signet(), 
                genesis_block_height: 0,
                //the regtest genesis block hash
                genesis_block_hash: bitcoin_move::bitcoin_hash::from_ascii_bytes(&b"0f9188f13cb7b2c71f2a335e3a4fc328bf5beb436012afca590b1a11466e2206"),
                reorg_block_count: 0,
                rooch_dao: MultisignAccountConfig{
                    multisign_bitcoin_address: rooch_framework::bitcoin_address::from_string(&std::string::utf8(b"bc1pevdrc8yqmgd94h2mpz9st0u77htmx935hzck3ruwsvcf4w7wrnqqd0yvze")),
                    threshold: 1,
                    participant_public_keys: vector[x"03ff7e1d7b4a152671124545f4fb68efe2a9bd0b3870ac22fee4afd4ecdfa8a19c"],
                }
            }
        );
        init();
    }

    #[test]
    fun test_init(){
        init_for_test();
        let network = network::network();
        assert!(network == network::network_signet(), 1000);
    }
}