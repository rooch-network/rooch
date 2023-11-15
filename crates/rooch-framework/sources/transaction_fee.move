// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transaction_fee {

    use moveos_std::context::{Self, Context};
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::coin::Coin;
    use rooch_framework::gas_coin::{GasCoin};

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    struct TransactionFeePool has key,store {
        fee: Object<CoinStore<GasCoin>>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, _genesis_account: &signer)  {
        let fee_store = coin_store::create_coin_store<GasCoin>(ctx);
        context::new_singleton(ctx, TransactionFeePool{
            fee: fee_store,
        });
    }

    /// Returns the gas factor of gas.
    public fun get_gas_factor(_ctx: &Context): u64 {
        //TODO we should provide a algorithm to cordanate the gas factor based on the network throughput
        return 1
    }

    public fun calculate_gas(ctx: &Context, gas_amount: u64): u256{
        (gas_amount as u256) * (get_gas_factor(ctx) as u256)
    }

    public(friend) fun deposit_fee(ctx: &mut Context, gas_coin: Coin<GasCoin>) {
        let object_id = object::singleton_object_id<TransactionFeePool>();
        let pool_object = context::borrow_mut_object_extend<TransactionFeePool>(ctx, object_id);
        let pool = object::borrow_mut(pool_object);
        coin_store::deposit<GasCoin>(&mut pool.fee, gas_coin);
    }
}
