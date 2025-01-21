// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_dex::liquidity_incentive {

    use std::signer::address_of;
    use std::u64;
    use moveos_std::bag::add;
    use app_admin::admin::AdminCap;
    use moveos_std::table;
    use rooch_framework::coin;
    use moveos_std::table::{Table, new};
    use moveos_std::signer;
    use rooch_framework::account_coin_store;
    use moveos_std::timestamp::now_seconds;
    use rooch_dex::swap_utils;
    use rooch_dex::swap::LPToken;
    use rooch_framework::coin_store;
    use rooch_framework::coin_store::CoinStore;
    use moveos_std::object::Object;
    use moveos_std::object;
    use rooch_framework::coin::Coin;

    #[test_only]
    use std::option::none;
    #[test_only]
    use std::string::utf8;
    #[test_only]
    use moveos_std::account::create_account_for_testing;
    #[test_only]
    use moveos_std::object::to_shared;
    #[test_only]
    use moveos_std::timestamp;

    #[test_only]
    use rooch_dex::swap::{init_lp_for_test, TestCoinX, TestCoinY};

    const ErrorFarmingNotStillFreeze: u64 = 1;
    const ErrorFarmingTotalWeightIsZero: u64 = 2;
    const ErrorExpDivideByZero: u64 = 3;
    const ErrorFarmingNotEnoughAsset: u64 = 4;
    const ErrorFarmingTimestampInvalid: u64 = 5;
    const ErrorFarmingCalcLastIdxBiggerThanNow: u64 = 6;
    const ErrorFarmingNotAlive: u64 = 7;
    const ErrorFarmingAliveStateInvalid: u64 = 8;
    const ErrorFarmingNotStake: u64 = 9;
    const ErrorNotCreator: u64 = 10;

    const EXP_MAX_SCALE: u128 = 9;

    //////////////////////////////////////////////////////////////////////
    // Exponential functions

    const EXP_SCALE: u128 = 1000000000000000000;// e18

    struct Exp has copy, store, drop {
        mantissa: u128
    }

    fun exp_direct(num: u128): Exp {
        Exp {
            mantissa: num
        }
    }

    fun exp_direct_expand(num: u128): Exp {
        Exp {
            mantissa: mul_u128(num, EXP_SCALE)
        }
    }


    fun mantissa(a: Exp): u128 {
        a.mantissa
    }

    fun add_exp(a: Exp, b: Exp): Exp {
        Exp {
            mantissa: add_u128(a.mantissa, b.mantissa)
        }
    }

    fun exp(num: u128, denom: u128): Exp {
        // if overflow move will abort
        let scaledNumerator = mul_u128(num, EXP_SCALE);
        let rational = div_u128(scaledNumerator, denom);
        Exp {
            mantissa: rational
        }
    }

    fun add_u128(a: u128, b: u128): u128 {
        a + b
    }

    fun sub_u128(a: u128, b: u128): u128 {
        a - b
    }

    fun mul_u128(a: u128, b: u128): u128 {
        if (a == 0 || b == 0) {
            return 0
        };
        a * b
    }

    fun div_u128(a: u128, b: u128): u128 {
        if (b == 0) {
            abort ErrorExpDivideByZero
        };
        if (a == 0) {
            return 0
        };
        a / b
    }

    fun truncate(exp: Exp): u128 {
        return exp.mantissa / EXP_SCALE
    }


    struct FarmingAsset<phantom X: key+store, phantom Y: key+ store, phantom RewardToken: key+store> has key, store {
        creator: address,
        asset_total_weight: u128,
        harvest_index: u128,
        last_update_timestamp: u64,
        // Release count per seconds
        release_per_second: u128,
        // Start time, by seconds, user can operate stake only after this timestamp
        start_time: u64,
        end_time: u64,
        coin_store: Object<CoinStore<RewardToken>>,
        stake_info: Table<address, Stake<X, Y>>,
        // Representing the pool is alive, false: not alive, true: alive.
        alive: bool,
    }

    /// To store user's asset token
    struct Stake<phantom X: key+store, phantom Y: key+ store> has key, store {
        asset: Object<CoinStore<LPToken<X, Y>>>,
        asset_weight: u128,
        last_harvest_index: u128,
        gain: u128,
    }

    public entry fun create_pool<X: key+store, Y: key+store, RewardToken: key+store>(
        account: &signer,
        release_per_second: u128,
        coin_amount: u256,
        start_time: u64,
    ){
        let reward_coin = account_coin_store::withdraw<RewardToken>(account, coin_amount);
        create_pool_with_coin<X, Y, RewardToken>(account, release_per_second, reward_coin, start_time)
    }

    public entry fun add_incentive<X: key+store, Y: key+store, RewardToken: key+store>(
        account: &signer,
        farming_asset_obj: &mut Object<FarmingAsset<X, Y, RewardToken>>,
        coin_amount: u256,
    ){
        let reward_coin = account_coin_store::withdraw<RewardToken>(account, coin_amount);
        let farming_asset = object::borrow_mut(farming_asset_obj);
        coin_store::deposit(&mut farming_asset.coin_store, reward_coin)
    }

    public entry fun withdraw_incentive<X: key+store, Y: key+store, RewardToken: key+store>(
        account: &signer,
        farming_asset_obj: &mut Object<FarmingAsset<X, Y, RewardToken>>,
        coin_amount: u256,
    ){

        let reward_coin = account_coin_store::withdraw<RewardToken>(account, coin_amount);
        let farming_asset = object::borrow_mut(farming_asset_obj);
        assert!(address_of(account) == farming_asset.creator, ErrorNotCreator);
        coin_store::deposit(&mut farming_asset.coin_store, reward_coin)
    }


    /// Add asset pools
    public fun create_pool_with_coin<X: key+store, Y: key+store, RewardToken: key+store>(
        account: &signer,
        release_per_second: u128,
        coin: Coin<RewardToken>,
        start_time: u64,
    ) {
        let end_time = (coin::value(&coin) / (release_per_second as u256) as u64);
        let coin_store = coin_store::create_coin_store<RewardToken>();
        coin_store::deposit(&mut coin_store, coin);
        if (swap_utils::sort_token_type<X, Y>()) {
            let farming_asset = object::new(FarmingAsset<X, Y, RewardToken> {
                creator: address_of(account),
                asset_total_weight: 0,
                harvest_index: 0,
                last_update_timestamp: start_time,
                release_per_second,
                start_time,
                end_time,
                coin_store,
                stake_info: new(),
                alive: true
            });
            object::to_shared(farming_asset)
        }else {
            let farming_asset = object::new(FarmingAsset<Y, X, RewardToken> {
                creator: address_of(account),
                asset_total_weight: 0,
                harvest_index: 0,
                last_update_timestamp: start_time,
                release_per_second,
                start_time,
                end_time,
                coin_store,
                stake_info: new(),
                alive: true
            });
            object::to_shared(farming_asset)
        };
    }

    /// Call by stake user, staking amount of asset in order to get yield farming token
    public entry fun stake<X: key+store, Y: key+store, RewardToken: key+store>(
        signer: &signer,
        lp_amount: u256,
        farming_asset_obj: &mut Object<FarmingAsset<X, Y, RewardToken>>,
    ) {
        let lp_token = account_coin_store::withdraw<LPToken<X, Y>>(signer, lp_amount);
        do_stake(signer, lp_token, farming_asset_obj);

    }

    public fun do_stake<X: key+store, Y: key+store, RewardToken: key+store>(
        signer: &signer,
        asset: Coin<LPToken<X, Y>>,
        farming_asset_obj: &mut Object<FarmingAsset<X, Y, RewardToken>>,
    ) {
        let account = signer::address_of(signer);
        let asset_weight = (coin::value(&asset) as u128);
        let farming_asset = object::borrow_mut<FarmingAsset<X,Y,RewardToken>>(farming_asset_obj);
        assert!(farming_asset.alive, ErrorFarmingNotAlive);

        // Check locking time
        let now_seconds = now_seconds();
        assert!(farming_asset.start_time <= now_seconds, ErrorFarmingNotStillFreeze);
        assert!(farming_asset.end_time > now_seconds, ErrorFarmingNotStillFreeze);

        let time_period = now_seconds - farming_asset.last_update_timestamp;

        if (farming_asset.asset_total_weight <= 0) {
            // Stake as first user
            let gain = farming_asset.release_per_second * (time_period as u128);
            let asset_coin_store = coin_store::create_coin_store<LPToken<X, Y>>();
            coin_store::deposit(&mut asset_coin_store, asset);
            table::add(&mut farming_asset.stake_info, account, Stake<X, Y> {
                asset: asset_coin_store,
                asset_weight,
                last_harvest_index: 0,
                gain,
            });
            farming_asset.harvest_index = 0;
            farming_asset.asset_total_weight = asset_weight;
        } else {
            let new_harvest_index = calculate_harvest_index_with_asset<X, Y, RewardToken>(farming_asset, now_seconds);
            if (!table::contains(&farming_asset.stake_info, account)) {
                let asset_coin_store = coin_store::create_coin_store<LPToken<X, Y>>();
                coin_store::deposit(&mut asset_coin_store, asset);
                table::add(&mut farming_asset.stake_info, account, Stake<X, Y> {
                    asset: asset_coin_store,
                    asset_weight,
                    last_harvest_index: new_harvest_index,
                    gain: 0,
                });
            }else {
                let stake = table::borrow_mut(&mut farming_asset.stake_info, account);
                let period_gain = calculate_withdraw_amount(
                    new_harvest_index,
                    stake.last_harvest_index,
                    stake.asset_weight
                );
                stake.gain = stake.gain + period_gain;
                stake.asset_weight = stake.asset_weight + asset_weight;
                stake.last_harvest_index = new_harvest_index;
                coin_store::deposit(&mut stake.asset, asset)
            };
            farming_asset.asset_total_weight = farming_asset.asset_total_weight + asset_weight;
            farming_asset.harvest_index = new_harvest_index;
        };
        farming_asset.last_update_timestamp = now_seconds;
    }

    /// Unstake asset from farming pool
    public entry fun unstake<X: key+store, Y: key+store, RewardToken: key+store>(signer: &signer, farming_asset_obj: &mut Object<FarmingAsset<X, Y, RewardToken>>) {
        let (lp_token, reward_token) = do_unstake(signer, farming_asset_obj);
        account_coin_store::deposit(address_of(signer), lp_token);
        account_coin_store::deposit(address_of(signer), reward_token);
    }

    public fun do_unstake<X: key+store, Y: key+store, RewardToken: key+store>(
        signer: &signer,
        farming_asset_obj: &mut Object<FarmingAsset<X, Y, RewardToken>>
    ): (Coin<LPToken<X,Y>>, Coin<RewardToken>){

        let farming_asset = object::borrow_mut(farming_asset_obj);
        assert!(table::contains(&farming_asset.stake_info, signer::address_of(signer)), ErrorFarmingNotStake);
        let Stake { last_harvest_index, asset_weight, asset, gain } =
            table::remove(&mut farming_asset.stake_info, signer::address_of(signer));

        let now_seconds = u64::min(now_seconds(), farming_asset.end_time);
        let new_harvest_index = calculate_harvest_index_with_asset(farming_asset, now_seconds);

        let period_gain = calculate_withdraw_amount(new_harvest_index, last_harvest_index, asset_weight);
        let total_gain = gain + period_gain;
        let withdraw_token = coin_store::withdraw(&mut farming_asset.coin_store, (total_gain as u256));

        assert!(farming_asset.asset_total_weight >= asset_weight, ErrorFarmingNotEnoughAsset);

        // Update farm asset
        farming_asset.asset_total_weight = farming_asset.asset_total_weight - asset_weight;

        farming_asset.last_update_timestamp = now_seconds;

        if (farming_asset.alive) {
            farming_asset.harvest_index = new_harvest_index;
        };

        (coin_store::remove_coin_store(asset), withdraw_token)
    }

    /// Harvest yield farming token from stake
    public entry fun harvest<X: key+store, Y: key+store, RewardToken: key+store>(
        signer: &signer,
        farming_asset_obj: &mut Object<FarmingAsset<X, Y, RewardToken>>
    ) {
        let reward_token = do_harvest(signer, farming_asset_obj);
        account_coin_store::deposit(address_of(signer), reward_token);
    }

    public fun do_harvest<X: key+store, Y: key+store, RewardToken: key+store>(
        signer: &signer,
        farming_asset_obj: &mut Object<FarmingAsset<X, Y, RewardToken>>
    ): Coin<RewardToken> {
        let farming_asset = object::borrow_mut(farming_asset_obj);
        assert!(table::contains(&farming_asset.stake_info, signer::address_of(signer)), ErrorFarmingNotStake);
        let now_seconds = u64::min(now_seconds(), farming_asset.end_time);
        let new_harvest_index = calculate_harvest_index_with_asset(farming_asset, now_seconds);
        let stake = table::borrow_mut(&mut farming_asset.stake_info, signer::address_of(signer));
        let period_gain = calculate_withdraw_amount(
            new_harvest_index,
            stake.last_harvest_index,
            stake.asset_weight
        );

        let total_gain = stake.gain + period_gain;

        let withdraw_token = coin_store::withdraw(&mut farming_asset.coin_store, (total_gain as u256));
        stake.gain = 0;
        stake.last_harvest_index = new_harvest_index;

        if (farming_asset.alive) {
            farming_asset.harvest_index = new_harvest_index;
        };
        farming_asset.last_update_timestamp = now_seconds;

        withdraw_token
    }

    /// The user can quering all yield farming amount in any time and scene
    public fun query_gov_token_amount<X: key+store, Y: key+store, RewardToken: key+store>(
        account: address,
        farming_asset_obj: &Object<FarmingAsset<X, Y, RewardToken>>
    ): u128 {
        let farming_asset = object::borrow<FarmingAsset<X, Y, RewardToken>>(farming_asset_obj);
        if (!table::contains(&farming_asset.stake_info, account)){
            return 0
        };
        let stake = table::borrow(&farming_asset.stake_info, account);
        let now_seconds = now_seconds();

        let new_harvest_index = calculate_harvest_index_with_asset(
            farming_asset,
            now_seconds
        );

        let new_gain = calculate_withdraw_amount(
            new_harvest_index,
            stake.last_harvest_index,
            stake.asset_weight
        );
        stake.gain + new_gain
    }

    /// Query total stake count from yield farming resource
    public fun query_total_stake<X: key+store, Y: key+store, RewardToken: key+store>(farming_asset_obj: &Object<FarmingAsset<X, Y, RewardToken>>): u128{
        let farming_asset = object::borrow<FarmingAsset<X, Y, RewardToken>>(farming_asset_obj);
        farming_asset.asset_total_weight
    }

    /// Query stake weight from user staking objects.
    public fun query_stake<X: key+store, Y: key+store, RewardToken: key+store>(farming_asset_obj: &Object<FarmingAsset<X, Y, RewardToken>>, account: address): u128 {
        let farming_asset = object::borrow<FarmingAsset<X, Y, RewardToken>>(farming_asset_obj);
        if (!table::contains(&farming_asset.stake_info, account)){
            return 0
        };
        let stake = table::borrow(&farming_asset.stake_info, account);
        stake.asset_weight
    }

    /// Queyry pool info from pool type
    /// return value: (alive, release_per_second, asset_total_weight, harvest_index)
    public fun query_info<X: key+store, Y: key+store, RewardToken: key+store>(farming_asset_obj: &Object<FarmingAsset<X, Y, RewardToken>>): (bool, u128, u128, u128) {
        let asset = object::borrow<FarmingAsset<X, Y, RewardToken>>(farming_asset_obj);
        (
            asset.alive,
            asset.release_per_second,
            asset.asset_total_weight,
            asset.harvest_index
        )
    }

    /// Update farming asset
    fun calculate_harvest_index_with_asset<X: key+store, Y: key+store, RewardToken: key+store>(
        farming_asset: &FarmingAsset<X, Y, RewardToken>,
        now_seconds: u64
    ): u128 {
        // Recalculate harvest index
        if (farming_asset.asset_total_weight <= 0) {
            calculate_harvest_index_weight_zero(
                farming_asset.harvest_index,
                farming_asset.last_update_timestamp,
                now_seconds,
                farming_asset.release_per_second
            )
        } else {
            calculate_harvest_index(
                farming_asset.harvest_index,
                farming_asset.asset_total_weight,
                farming_asset.last_update_timestamp,
                now_seconds,
                farming_asset.release_per_second
            )
        }
    }

    /// There is calculating from harvest index and global parameters without asset_total_weight
    public fun calculate_harvest_index_weight_zero(
        harvest_index: u128,
        last_update_timestamp: u64,
        now_seconds: u64,
        release_per_second: u128
    ): u128 {
        assert!(last_update_timestamp <= now_seconds, ErrorFarmingTimestampInvalid);
        let time_period = now_seconds - last_update_timestamp;
        let addtion_index = release_per_second * ((time_period as u128));
        harvest_index + mantissa(exp_direct_expand(addtion_index))
    }

    /// There is calculating from harvest index and global parameters
    public fun calculate_harvest_index(
        harvest_index: u128,
        asset_total_weight: u128,
        last_update_timestamp: u64,
        now_seconds: u64,
        release_per_second: u128
    ): u128 {
        assert!(asset_total_weight > 0, ErrorFarmingTotalWeightIsZero);
        assert!(last_update_timestamp <= now_seconds, ErrorFarmingTimestampInvalid);

        let time_period = now_seconds - last_update_timestamp;
        let numr = (release_per_second * (time_period as u128));
        let denom = asset_total_weight;
        harvest_index + mantissa(exp(numr, denom))
    }

    /// This function will return a gain index
    public fun calculate_withdraw_amount(
        harvest_index: u128,
        last_harvest_index: u128,
        asset_weight: u128
    ): u128 {
        assert!(
            harvest_index >= last_harvest_index,
            ErrorFarmingCalcLastIdxBiggerThanNow
        );
        let amount = asset_weight * (harvest_index - last_harvest_index);
        truncate(exp_direct(amount))
    }

    #[test_only]
    struct TestRewardCoin has key, store{}

    #[test(sender=@0x42)]
    fun test_stake(sender: signer) {
        let account_b = create_account_for_testing(@0x43);
        rooch_framework::genesis::init_for_test();
        let lp_reward_info = coin::register_extend<TestRewardCoin>(utf8(b"Test Lp Reward Coin"), utf8(b"TLR"), none(), 8);
        let lp_reward_coin = coin::mint(&mut lp_reward_info, 100000);
        to_shared(lp_reward_info);
        let lp_coin = init_lp_for_test(10000);
        let lp_coin2 = coin::extract(&mut lp_coin, 5000);
        account_coin_store::deposit(address_of(&sender), lp_coin);
        account_coin_store::deposit(address_of(&account_b), lp_coin2);
        let coin_store = coin_store::create_coin_store<TestRewardCoin>();
        coin_store::deposit(&mut coin_store, lp_reward_coin);
        let farming_asset_obj = object::new(
            FarmingAsset<TestCoinX, TestCoinY, TestRewardCoin> {
                creator: address_of(&sender),
                asset_total_weight: 0,
                harvest_index: 0,
                last_update_timestamp: now_seconds(),
                release_per_second: 100,
                start_time:now_seconds(),
                end_time: now_seconds() + 10000,
                coin_store,
                stake_info: new(),
                alive: true
            });
        stake(&sender, 100, &mut farming_asset_obj);
        let seconds = 100;
        timestamp::fast_forward_seconds_for_test(seconds);
        let reward_a = query_gov_token_amount(address_of(&sender), &farming_asset_obj);
        stake(&account_b, 100, &mut farming_asset_obj);
        let total_weight = query_total_stake(&farming_asset_obj);
        assert!(total_weight == 200, 1);
        assert!(reward_a == 10000, 2);
        timestamp::fast_forward_seconds_for_test(seconds);
        reward_a = query_gov_token_amount(address_of(&sender), &farming_asset_obj);
        assert!(reward_a == 15000, 3);
        let reward_b = query_gov_token_amount(address_of(&account_b), &farming_asset_obj);
        assert!(reward_b == 5000, 4);
        let reward_coin = do_harvest(&sender, &mut farming_asset_obj);
        assert!(coin::value(&reward_coin) == 15000, 5);
        account_coin_store::deposit(address_of(&sender), reward_coin);
        to_shared(farming_asset_obj);


    }
}