module rooch_framework::coin_mapping{
    
    use std::option::{Self, Option};
    use std::signer;
    use std::string;
    use moveos_std::type_info;
    use rooch_framework::address_mapping::MultiChainAddress;
    use rooch_framework::coin::CoinInfo;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::table::{Self, Table};
    use moveos_std::account_storage;
    use moveos_std::signer as moveos_signer;

    //The coin id standard is defined in [slip-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
    //Please keep consistent with rust CoinID
    const COIN_TYPE_BTC: u64 = 0;
    const COIN_TYPE_ETH: u64 = 60;
    const COIN_TYPE_NOSTR: u64 = 1237;
    const COIN_TYPE_ROH: u64 = 20230101;

    // struct MultiChainCoin has copy, store, drop {
    //     coin_id: u64,
    //     raw_coin: vector<u8>,
    // }
    struct MultiChainCoin has copy, store, drop {
        coin_id: u64,
        maddress: MultiChainAddress,
        /// Symbol of the coin, usually a shorter version of the name.
        /// For example, Singapore Dollar is SGD.
        symbol: string::String,
    }

    // ETH -> rooch ETH
    struct RoochCoin has copy, store, drop {
        coin_type: type_info::TypeInfo,
        /// Symbol of the coin, usually a shorter version of the name.
        /// For example, Singapore Dollar is SGD.
        symbol: string::String,
    }

    
    // struct CoinMapping has key{
    //     mapping: Table<MultiChainCoin, coin>,
    // }

    struct CoinMapping has key{
        mapping: Table<MultiChainCoin, RoochCoin>,
    }

    struct CoinInfos has key{
        mapping: Table<MultiChainCoin, RoochCoin>,
    }


    fun init(ctx: &mut StorageContext) {
        let sender = &moveos_signer::module_signer<CoinMapping>();
        rooch_framework::core_coines::assert_rooch_framework(sender);
        let tx_ctx = storage_context::tx_context_mut(ctx);
        let mapping = table::new<MultiChainCoin, RoochCoin>(tx_ctx);
        account_storage::global_move_to(ctx, sender, CoinMapping{
            mapping,
        });
    }

    public fun is_rooch_coin(mcoin: &MultiChainCoin) : bool{
        mcoin.coin_id == COIN_TYPE_ROH
    }

    /// Resolve a multi-chain coin to a rooch coin
    public fun resolve(ctx: &StorageContext, mcoin: MultiChainCoin): Option<RoochCoin> {
        if (is_rooch_coin(&mcoin)) {
            return option::some(moveos_std::bcs::to_coin(mcoin.raw_coin))
        };
        let cm = account_storage::global_borrow<CoinMapping>(ctx, @rooch_framework);
        if(table::contains(&cm.mapping, mcoin)){
            let coin = table::borrow(&cm.mapping, mcoin);
            option::some( *coin )
        }else{
            option::none()
        }
    }

    /// Resolve a multi-chain coin to a rooch coin, if not exists, generate a new rooch coin
    public fun resolve_or_generate(ctx: &StorageContext, mcoin: MultiChainCoin): coin {
        let coin = resolve(ctx, mcoin);
        if(option::is_none( &coin )){
            generate_rooch_coin(mcoin)
        }else{
            option::extract( &mut coin )
        }
    }
    
    // fun generate_rooch_coin(mcoin: MultiChainCoin): coin {
    //     let hash = blake2b256(&mcoin.raw_coin);
    //     moveos_std::bcs::to_coin(hash)
    // }

    /// Check if a multi-chain coin is bound to a rooch coin
    public fun exists_mapping(ctx: &StorageContext, mcoin: MultiChainCoin): bool {
        if (is_rooch_coin(&mcoin)) {
            return true
        };
        let cm = account_storage::global_borrow<CoinMapping>(ctx, @rooch_framework);
        table::contains(&cm.mapping, mcoin)
    }

    /// Bind a multi-chain coin to the sender's rooch coin
    /// The caller need to ensure the relationship between the multi-chain coin and the rooch coin
    public fun bind(ctx: &mut StorageContext, sender: &signer, mcoin: MultiChainCoin) {
        bind_no_check(ctx, signer::address_of(sender), mcoin);
    } 

    /// Bind a rooch coin to a multi-chain coin
    public(friend) fun bind_no_check(ctx: &mut StorageContext, rooch_coin: coin, mcoin: MultiChainCoin) {
        if(is_rooch_coin(&mcoin)){
            //Do nothing if the multi-chain coin is a rooch coin
            return
        };
        let cm = account_storage::global_borrow_mut<CoinMapping>(ctx, @rooch_framework);
        table::add(&mut cm.mapping, mcoin, rooch_coin);
        //TODO matienance the reverse mapping rooch_coin -> vector<MultiChainCoin>
    }

    #[test(sender=@rooch_framework)]
    fun test_coin_mapping(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        account_storage::create_account_storage(&mut ctx, @rooch_framework);
        init(&mut ctx);
        let multi_chain_coin =  MultiChainCoin{
            coin_id: COIN_TYPE_BTC,
            raw_coin: x"1234567890abcdef",
        };
        bind(&mut ctx, &sender, multi_chain_coin);
        let coin = option::extract(&mut resolve(&ctx, multi_chain_coin));
        assert!(addr == @rooch_framework, 1000);
        storage_context::drop_test_context(ctx);
    }
}