// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// The transaction fee module is used to manage the transaction fee pool.
/// Distribution of Transaction Gas Fees:
/// 
/// 1. RoochNetwork 40%
///     * Before Mainnet launch: Used to repay the debt from Gas airdrops
///     * After Mainnet launch: Used to buy back Mainnet tokens
/// 2. Sequencer 30%
/// 3. Application Developers 30%
///     * Goes to the developer of the entry function contract called by the transaction
///     * If the entry contract is a system Framework contract, this portion goes to the Rooch network

module rooch_framework::transaction_fee {

    use moveos_std::object::{Self, Object};
    use moveos_std::core_addresses;
    use moveos_std::signer;

    use rooch_framework::coin_store::{Self, CoinStore};
    use rooch_framework::coin::{Self,Coin};
    use rooch_framework::gas_coin::{RGas};
    use rooch_framework::account_coin_store;

    friend rooch_framework::genesis;
    friend rooch_framework::transaction_validator;

    const SystemFeeAddress: address = @rooch_framework;
    
    ///Error code for invalid gas used in transaction
    const ErrorInvalidGasUsed: u64 = 1;

    struct TransactionFeePool has key {
        fee: Object<CoinStore<RGas>>,
    }

    public(friend) fun genesis_init(_genesis_account: &signer)  {
        let fee_store = coin_store::create_coin_store<RGas>();
        let obj = object::new_named_object(TransactionFeePool{
            fee: fee_store,
        });
        object::transfer_extend(obj, @rooch_framework);
    }

    // Borrow the gas revenue store or init a new one if it does not exist for the given address.
    // The address can be a contract address or sequencer address.
    fun borrow_mut_or_init_gas_revenue_store(addr: address): &mut Object<CoinStore<RGas>>{
        let fee_pool_id = object::named_object_id<TransactionFeePool>();
        let fee_pool_object = object::borrow_mut_object_extend<TransactionFeePool>(fee_pool_id);
        if(!object::contains_field(fee_pool_object, addr)){
            let gas_revenue_store_obj = coin_store::create_coin_store<RGas>();
            object::add_field(fee_pool_object, addr, gas_revenue_store_obj);
        };
        object::borrow_mut_field(fee_pool_object, addr)
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

    public(friend) fun withdraw_fee(amount: u256) : Coin<RGas> {
        let object_id = object::named_object_id<TransactionFeePool>();
        let pool_object = object::borrow_mut_object_extend<TransactionFeePool>(object_id);
        let pool = object::borrow_mut(pool_object);
        coin_store::withdraw<RGas>(&mut pool.fee, amount)
    }

    public(friend) fun deposit_fee(gas_coin: Coin<RGas>) {
        let object_id = object::named_object_id<TransactionFeePool>();
        let pool_object = object::borrow_mut_object_extend<TransactionFeePool>(object_id);
        let pool = object::borrow_mut(pool_object);
        coin_store::deposit<RGas>(&mut pool.fee, gas_coin);
    }

    public(friend) fun distribute_fee(total_paid_gas: u256, gas_used: u256, contract_address: address, sequencer_address: address) : Coin<RGas> {
        assert!(total_paid_gas >= gas_used, ErrorInvalidGasUsed);
        let total_paid_gas_coin = withdraw_fee(total_paid_gas);
        let used_gas_coin = coin::extract(&mut total_paid_gas_coin, gas_used);
        
        let sequencer_fee = gas_used * 30 / 100;
        let developer_fee = gas_used * 30 / 100;
        
        let sequencer_fee_coin = coin::extract(&mut used_gas_coin, sequencer_fee);
        let sequencer_fee_coin_store = borrow_mut_or_init_gas_revenue_store(sequencer_address);
        coin_store::deposit(sequencer_fee_coin_store, sequencer_fee_coin);

        let is_framework_address = core_addresses::is_system_reserved_address(contract_address);
        if(!is_framework_address){
            let developer_fee_coin = coin::extract(&mut used_gas_coin, developer_fee);
            let developer_fee_coin_store = borrow_mut_or_init_gas_revenue_store(contract_address);
            coin_store::deposit(developer_fee_coin_store, developer_fee_coin);
        };

        let system_fee_coin_store = borrow_mut_or_init_gas_revenue_store(SystemFeeAddress);
        coin_store::deposit(system_fee_coin_store, used_gas_coin);
        //Return the remaining gas coin to the sender
        total_paid_gas_coin
    }

    /// Withdraw the gas revenue for the sender
    /// The contract address can use `moveos_std::signer::module_signer` to get the signer
    public fun withdraw_gas_revenue(sender: &signer, amount: u256): Coin<RGas> {
        let addr = signer::address_of(sender);
        let gas_revenue_store = borrow_mut_or_init_gas_revenue_store(addr);
        coin_store::withdraw(gas_revenue_store, amount)
    }

    /// The entry function to withdraw the gas revenue for the sender
    public entry fun withdraw_gas_revenue_entry(sender: &signer, amount: u256){
        let coin = withdraw_gas_revenue(sender, amount);
        account_coin_store::deposit(signer::address_of(sender), coin);
    }

    /// Get the gas revenue balance for the given address
    public fun gas_revenue_balance(addr: address): u256 {
        let fee_pool_id = object::named_object_id<TransactionFeePool>();
        let fee_pool_object = object::borrow_object<TransactionFeePool>(fee_pool_id);
        if(object::contains_field(fee_pool_object, addr)){
            let gas_revenue_store = object::borrow_field<TransactionFeePool, address, Object<CoinStore<RGas>>>(fee_pool_object, addr);
            coin_store::balance(gas_revenue_store)
        }else{
            0u256
        }
    }

    #[test]
    fun test_distribute_fee(){
        let system_signer = moveos_std::account::create_signer_for_testing(SystemFeeAddress);   
        rooch_framework::coin::init_for_testing();
        rooch_framework::gas_coin::genesis_init(&system_signer);
        genesis_init(&system_signer);

        let gas_coin = rooch_framework::gas_coin::mint_for_test(120);
        deposit_fee(gas_coin);
        let total_paid_gas = 120;
        let gas_used = 100;
        let contract_address = @0x42;
        let sequencer_address = @0x43;
        let remaining_gas_coin = distribute_fee(total_paid_gas, gas_used, contract_address, sequencer_address);
        assert!(coin::value(&remaining_gas_coin) == 20, 1);
        
        let contract_signer = moveos_std::account::create_signer_for_testing(contract_address);
        let gas_revenue = withdraw_gas_revenue(&contract_signer, 30);
        assert!(coin::value(&gas_revenue) == 30, 2);
        
        let sequencer_signer = moveos_std::account::create_signer_for_testing(sequencer_address);
        let sequencer_gas_revenue = withdraw_gas_revenue(&sequencer_signer, 30);
        assert!(coin::value(&sequencer_gas_revenue) == 30, 3);

        
        let system_gas_revenue = withdraw_gas_revenue(&system_signer, 40);
        assert!(coin::value(&system_gas_revenue) == 40, 4);
        
        coin::destroy_for_testing(remaining_gas_coin);
        coin::destroy_for_testing(gas_revenue);
        coin::destroy_for_testing(sequencer_gas_revenue);
        coin::destroy_for_testing(system_gas_revenue);
    }
}
