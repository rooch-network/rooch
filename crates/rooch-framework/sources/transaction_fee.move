// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_framework::transaction_fee {

    use moveos_std::account_storage;
    use moveos_std::context::Context;
    use moveos_std::object_ref::{Self, ObjectRef};
    use rooch_framework::coin::{Self, Coin, CoinStore};
    use rooch_framework::gas_coin::{GasCoin};

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    struct TransactionFeePool has key,store {
        fee: ObjectRef<CoinStore>,
    }

    public(friend) fun genesis_init(ctx: &mut Context, genesis_account: &signer)  {
        let fee_store = coin::create_coin_store<GasCoin>(ctx);
        account_storage::global_move_to(ctx, genesis_account, TransactionFeePool{
            fee: fee_store,
        })
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
        let pool = account_storage::global_borrow_mut<TransactionFeePool>(ctx, @rooch_framework);
        let coin_store = object_ref::borrow_mut(&mut pool.fee); 
        coin::deposit_to_store<GasCoin>(coin_store, gas_coin);
    }
}
