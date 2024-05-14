//# init --addresses test=0x42 test2=0x43

//check the account coin store object id
//# run --signers test
script {
    
    use rooch_framework::coin_store::CoinStore;
    use rooch_framework::gas_coin::GasCoin;

    fun main(sender: &signer) {
        let account_addr = moveos_std::signer::address_of(sender);
        let object_id = moveos_std::object::account_named_object_id<CoinStore<GasCoin>>(account_addr);
        std::debug::print(&object_id);
        std::debug::print(&rooch_framework::coin::is_registered<GasCoin>());
        std::debug::print(&rooch_framework::account_coin_store::balance<GasCoin>(account_addr));
    }
}

//Get gas from faucet
//# run rooch_framework::gas_coin::faucet_entry --signers test --args u256:10000000000

//Transfer via coin store
//# run --signers test --args object:0xd073508b9582eff4e01078dc2e62489c15bbef91b6a2e568ac8fb33f0cf54daa
script {
    use moveos_std::object::{Object};
    
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::gas_coin::{Self, GasCoin};

    fun main(coin_store: &mut Object<CoinStore<GasCoin>>) {
        let gas_coin = coin_store::withdraw(coin_store, 100);
        gas_coin::burn(gas_coin);
    }
}