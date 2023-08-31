module rooch_framework::transaction_fee {

    use moveos_std::account_storage;
    use moveos_std::storage_context::StorageContext;
    use rooch_framework::coin::{Self, Coin};
    use rooch_framework::gas_coin::{GasCoin};

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    struct TransactionFeePool has key,store {
        fee: Coin<GasCoin>,
    }

    public(friend) fun genesis_init(ctx: &mut StorageContext, genesis_account: &signer)  {
        account_storage::global_move_to(ctx, genesis_account, TransactionFeePool{
            fee: coin::zero<GasCoin>(),
        })
    }

    /// Returns the gas factor of gas.
    public fun get_gas_factor(_ctx: &StorageContext): u64 {
        //TODO we should provide a algorithm to cordanate the gas factor based on the network throughput
        return 1
    }

    public fun calculate_gas(ctx: &StorageContext, gas_amount: u64): u256{
        (gas_amount as u256) * (get_gas_factor(ctx) as u256)
    }

    public(friend) fun deposit_fee(ctx: &mut StorageContext, gas_coin: Coin<GasCoin>) {
        let pool = account_storage::global_borrow_mut<TransactionFeePool>(ctx, @rooch_framework);
        coin::merge(&mut pool.fee, gas_coin);
    }
}