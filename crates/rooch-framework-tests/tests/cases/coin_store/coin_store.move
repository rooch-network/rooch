//# init --addresses test=0x42 test2=0x43

//check the account coin store object id
//# run --signers test
script {
    
    use rooch_framework::coin_store::CoinStore;
    use rooch_framework::gas_coin::RGas;

    fun main(sender: &signer) {
        let account_addr = moveos_std::signer::address_of(sender);
        let object_id = moveos_std::object::account_named_object_id<CoinStore<RGas>>(account_addr);
        std::debug::print(&object_id);
        std::debug::print(&rooch_framework::coin::is_registered<RGas>());
        std::debug::print(&rooch_framework::account_coin_store::balance<RGas>(account_addr));
    }
}

//Get gas from faucet
//# run rooch_framework::gas_coin::faucet_entry --signers test --args u256:100000000000

//Transfer via coin store
//# run --signers test --args object:0x562409111a2ca55814e56eb42186470c4adda4a04a4a84140690f4d68e8e1c06
script {
    use moveos_std::object::{Object};
    
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::gas_coin::{Self, RGas};

    fun main(coin_store: &mut Object<CoinStore<RGas>>) {
        let gas_coin = coin_store::withdraw(coin_store, 100);
        gas_coin::burn(gas_coin);
    }
}