module rooch_framework::address_mapping{
    
    use std::option::{Self, Option};
    use std::signer;
    use std::error;
    use moveos_std::context::Context;
    use moveos_std::table::{Self, Table};
    use moveos_std::account_storage;
    use rooch_framework::hash::{blake2b256};
    use rooch_framework::multichain_address::{Self, MultiChainAddress};

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;
    friend rooch_framework::transfer;
    
    const ErrorMultiChainAddressInvalid: u64 = 1;

    struct AddressMapping has key{
        mapping: Table<MultiChainAddress, address>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, genesis_account: &signer) {
        let mapping = table::new<MultiChainAddress, address>(ctx);
        account_storage::global_move_to(ctx, genesis_account, AddressMapping{
            mapping,
        });
    }

    /// Resolve a multi-chain address to a rooch address
    public fun resolve(ctx: &Context, maddress: MultiChainAddress): Option<address> {
        if (multichain_address::is_rooch_address(&maddress)) {
            return option::some(multichain_address::into_rooch_address(maddress))
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
    public fun resolve_or_generate(ctx: &Context, maddress: MultiChainAddress): address {
        let addr = resolve(ctx, maddress);
        if(option::is_none(&addr)){
            generate_rooch_address(&maddress)
        }else{
            option::extract(&mut addr)
        }
    }
    
    fun generate_rooch_address(maddress: &MultiChainAddress): address {
        let hash = blake2b256(multichain_address::raw_address(maddress));
        moveos_std::bcs::to_address(hash)
    }

    /// Check if a multi-chain address is bound to a rooch address
    public fun exists_mapping(ctx: &Context, maddress: MultiChainAddress): bool {
        if (multichain_address::is_rooch_address(&maddress)) {
            return true
        };
        let am = account_storage::global_borrow<AddressMapping>(ctx, @rooch_framework);
        table::contains(&am.mapping, maddress)
    }

    /// Bind a multi-chain address to the sender's rooch address
    /// The caller need to ensure the relationship between the multi-chain address and the rooch address
    public fun bind(ctx: &mut Context, sender: &signer, maddress: MultiChainAddress) {
        bind_no_check(ctx, signer::address_of(sender), maddress);
    } 

    /// Bind a rooch address to a multi-chain address
    public(friend) fun bind_no_check(ctx: &mut Context, rooch_address: address, maddress: MultiChainAddress) {
        if(multichain_address::is_rooch_address(&maddress)){
            assert!(
                multichain_address::into_rooch_address(maddress) == rooch_address, 
                error::invalid_argument(ErrorMultiChainAddressInvalid)
            );
        };
        let am = account_storage::global_borrow_mut<AddressMapping>(ctx, @rooch_framework);
        table::add(&mut am.mapping, maddress, rooch_address);
        //TODO matienance the reverse mapping rooch_address -> vector<MultiChainAddress>
    }
   
}