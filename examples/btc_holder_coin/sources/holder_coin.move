// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module btc_holder_coin::holder_coin {

    use std::string;
    use moveos_std::timestamp;
    use moveos_std::tx_context;
    use moveos_std::object::{Self, Object};
    use rooch_framework::coin::{Self, Coin, CoinInfo};
    use rooch_framework::account_coin_store;
    use bitcoin_move::utxo::{Self, UTXO};

    /// The decimals of the `BTC Holder Coin`
    const DECIMALS: u8 = 1u8;

    const ErrorAlreadyStaked: u64 = 1;
    const ErrorAlreadyClaimed: u64 = 2;

    /// The `BTC Holder Coin`
    struct HDC has key, store {}

    /// Hold the CoinInfo object
    struct CoinInfoHolder has key {
        coin_info: Object<CoinInfo<HDC>>,
    }

    /// The stake info of UTXO
    /// This Info store in the temporary state area of UTXO
    /// If the UTXO is spent, the stake info will be removed
    struct StakeInfo has store, drop {
        start_time: u64,
        last_claim_time: u64,
    }

    fun init() {
        let coin_info_obj = coin::register_extend<HDC>(
            string::utf8(b"BTC Holder Coin"),
            string::utf8(b"HDC"),
            DECIMALS,
        );
        let coin_info_holder_obj = object::new_named_object(CoinInfoHolder { coin_info: coin_info_obj });
        // Make the coin info holder object to shared, so anyone can get mutable CoinInfoHolder object
        object::to_shared(coin_info_holder_obj);
    }

    /// Stake the UTXO to get the `BTC Holder Coin`
    public fun do_stake(utxo: &mut Object<UTXO>) {
        assert!(!utxo::contains_temp_state<StakeInfo>(utxo), ErrorAlreadyStaked);
        let now = timestamp::now_seconds();
        let stake_info = StakeInfo { start_time: now, last_claim_time: now};
        utxo::add_temp_state(utxo, stake_info);
    }

    /// Claim the `BTC Holder Coin` from the UTXO
    public fun do_claim(coin_info_holder_obj: &mut Object<CoinInfoHolder>, utxo_obj: &mut Object<UTXO>): Coin<HDC> {
        let utxo_value = utxo::value(object::borrow(utxo_obj));
        let stake_info = utxo::borrow_mut_temp_state<StakeInfo>(utxo_obj);
        let now = timestamp::now_seconds();
        assert!(stake_info.last_claim_time < now, ErrorAlreadyClaimed);
        let coin_info_holder = object::borrow_mut(coin_info_holder_obj);
        let mint_amount = (((now - stake_info.last_claim_time) * utxo_value) as u256);
        let coin = coin::mint_extend(&mut coin_info_holder.coin_info, mint_amount);
        stake_info.last_claim_time = now;
        coin
    }

    public entry fun stake(utxo: &mut Object<UTXO>){
       do_stake(utxo);
    }

    public entry fun claim(coin_info_holder_obj: &mut Object<CoinInfoHolder>, utxo: &mut Object<UTXO>) {
        let coin = do_claim(coin_info_holder_obj, utxo);
        let sender = tx_context::sender();
        account_coin_store::deposit(sender, coin);
    }

    #[test]
    fun test_stake_claim() {
        bitcoin_move::genesis::init_for_test();
        init();
        let seconds = 100;
        let tx_id = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let sat_value = 100000000;
        let utxo = utxo::new_for_testing(tx_id, 0u32, sat_value);
        do_stake(&mut utxo);
        timestamp::fast_forward_seconds_for_test(seconds);
        let coin_info_holder_obj_id = object::named_object_id<CoinInfoHolder>();
        let coin_info_holder_obj = object::borrow_mut_object_shared<CoinInfoHolder>(coin_info_holder_obj_id);
        let hdc_coin = do_claim(coin_info_holder_obj, &mut utxo);
        let expected_coin_value = ((sat_value * seconds) as u256);
        assert!(coin::value(&hdc_coin) == expected_coin_value, 1000);
        coin::destroy_for_testing(hdc_coin);
        utxo::drop_for_testing(utxo);
    }
}