// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transaction_fee {

    use moveos_std::object::{Self, Object};
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::coin::Coin;
    use rooch_framework::gas_coin::{GasCoin};

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    struct TransactionFeePool has key {
        fee: Object<CoinStore<GasCoin>>,
    }

    public(friend) fun genesis_init(_genesis_account: &signer)  {
        let fee_store = coin_store::create_coin_store<GasCoin>();
        let obj = object::new_named_object(TransactionFeePool{
            fee: fee_store,
        });
        object::transfer_extend(obj, @rooch_framework);
    }

    /// Returns the gas factor of gas.
    public fun get_gas_factor(): u64 {
        //TODO we should provide a algorithm to cordanate the gas factor based on the network throughput
        //https://github.com/rooch-network/rooch/issues/1733
        return 1
    }

    public fun calculate_gas(gas_amount: u64): u256{
        (gas_amount as u256) * (get_gas_factor() as u256)
    }

    public(friend) fun deposit_fee(gas_coin: Coin<GasCoin>) {
        let object_id = object::named_object_id<TransactionFeePool>();
        let pool_object = object::borrow_mut_object_extend<TransactionFeePool>(object_id);
        let pool = object::borrow_mut(pool_object);
        coin_store::deposit<GasCoin>(&mut pool.fee, gas_coin);
    }
}
