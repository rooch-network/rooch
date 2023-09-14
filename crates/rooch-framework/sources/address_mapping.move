module rooch_framework::address_mapping{
    
    use std::option::{Self, Option};
    use std::signer;
    use moveos_std::storage_context::{Self, StorageContext};
    use moveos_std::table::{Self, Table};
    use moveos_std::account_storage;
    use moveos_std::signer as moveos_signer;
    use rooch_framework::hash::{blake2b256};

    friend rooch_framework::transaction_validator;

    //The multichain id standard is defined in [slip-0044](https://github.com/satoshilabs/slips/blob/master/slip-0044.md)
    //Please keep consistent with rust Symbol
    const MULTICHAIN_ID_BITCOIN: u64 = 0;
    const MULTICHAIN_ID_ETHER: u64 = 60;
    const MULTICHAIN_ID_NOSTR: u64 = 1237;
    const MULTICHAIN_ID_ROOCH: u64 = 20230103;

    struct MultiChainAddress has copy, store, drop {
        multichain_id: u64,
        raw_address: vector<u8>,
    }
    
    struct AddressMapping has key{
        mapping: Table<MultiChainAddress, address>,
    }

    fun init(ctx: &mut StorageContext) {
        let sender = &moveos_signer::module_signer<AddressMapping>();
        rooch_framework::core_addresses::assert_rooch_framework(sender);
        let tx_ctx = storage_context::tx_context_mut(ctx);
        let mapping = table::new<MultiChainAddress, address>(tx_ctx);
        account_storage::global_move_to(ctx, sender, AddressMapping{
            mapping,
        });
    }

    public fun is_rooch_address(maddress: &MultiChainAddress) : bool{
        maddress.multichain_id == MULTICHAIN_ID_ROOCH
    }

    /// Resolve a multi-chain address to a rooch address
    public fun resolve(ctx: &StorageContext, maddress: MultiChainAddress): Option<address> {
        if (is_rooch_address(&maddress)) {
            return option::some(moveos_std::bcs::to_address(maddress.raw_address))
        };
        let am = account_storage::global_borrow<AddressMapping>(ctx, @rooch_framework);
        if(table::contains(&am.mapping, maddress)){
            let addr = table::borrow(&am.mapping, maddress);
            option::some(*addr)
        }else{
            option::none()
        }
    }

    /// Resolve a multi-chain address to a rooch address, if not exists, generate a new rooch address
    public fun resolve_or_generate(ctx: &StorageContext, maddress: MultiChainAddress): address {
        let addr = resolve(ctx, maddress);
        if(option::is_none(&addr)){
            generate_rooch_address(maddress)
        }else{
            option::extract(&mut addr)
        }
    }
    
    fun generate_rooch_address(maddress: MultiChainAddress): address {
        let hash = blake2b256(&maddress.raw_address);
        moveos_std::bcs::to_address(hash)
    }

    /// Check if a multi-chain address is bound to a rooch address
    public fun exists_mapping(ctx: &StorageContext, maddress: MultiChainAddress): bool {
        if (is_rooch_address(&maddress)) {
            return true
        };
        let am = account_storage::global_borrow<AddressMapping>(ctx, @rooch_framework);
        table::contains(&am.mapping, maddress)
    }

    /// Bind a multi-chain address to the sender's rooch address
    /// The caller need to ensure the relationship between the multi-chain address and the rooch address
    public fun bind(ctx: &mut StorageContext, sender: &signer, maddress: MultiChainAddress) {
        bind_no_check(ctx, signer::address_of(sender), maddress);
    } 

    /// Bind a rooch address to a multi-chain address
    public(friend) fun bind_no_check(ctx: &mut StorageContext, rooch_address: address, maddress: MultiChainAddress) {
        if(is_rooch_address(&maddress)){
            //Do nothing if the multi-chain address is a rooch address
            return
        };
        let am = account_storage::global_borrow_mut<AddressMapping>(ctx, @rooch_framework);
        table::add(&mut am.mapping, maddress, rooch_address);
        //TODO matienance the reverse mapping rooch_address -> vector<MultiChainAddress>
    }

    #[test(sender=@rooch_framework)]
    fun test_address_mapping(sender: signer){
        let sender_addr = signer::address_of(&sender);
        let ctx = storage_context::new_test_context(sender_addr);
        account_storage::create_account_storage(&mut ctx, @rooch_framework);
        init(&mut ctx);
        let multi_chain_address = MultiChainAddress{
            multichain_id: MULTICHAIN_ID_BITCOIN,
            raw_address: x"1234567890abcdef",
        };
        bind(&mut ctx, &sender, multi_chain_address);
        let addr = option::extract(&mut resolve(&ctx, multi_chain_address));
        assert!(addr == @rooch_framework, 1000);
        storage_context::drop_test_context(ctx);
    }
}