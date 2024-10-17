// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module grow_bitcoin::grow_bitcoin {

    use std::string;
    use std::option;
    use std::u64;
    use bitcoin_move::bbn;
    use bitcoin_move::bbn::BBNStakeSeal;
    use bitcoin_move::types;
    use bitcoin_move::bitcoin;

    use moveos_std::event::emit;
    use moveos_std::tx_context::sender;
    use moveos_std::table;
    use moveos_std::table::Table;
    use moveos_std::object;
    use moveos_std::signer;
    use moveos_std::object::{Object, ObjectID};
    use moveos_std::timestamp;
    use moveos_std::account;
    use moveos_std::event_queue::{Self, Subscriber};
    use moveos_std::type_info;

    use rooch_framework::account_coin_store;
    use rooch_framework::coin;
    use rooch_framework::coin::{CoinInfo, Coin};
    
    use bitcoin_move::utxo::{Self, UTXO, value, TempStateDropEvent};
    #[test_only]
    use bitcoin_move::bitcoin::add_latest_block;

    use app_admin::admin::AdminCap;


    const DEPLOYER: address = @grow_bitcoin;

    const MaxLockDay: u64 = 1000;
    // 1 Day seconds
    const PerDaySeconds: u32 = 86400;
    // 1 Day blocks
    const PerDayBlocks: u64 = 144;
    // LockTime values below the threshold are interpreted as block heights
    // values above (or equal to) the threshold are interpreted as block times (UNIX timestamp, seconds since epoch).
    const LOCK_TIME_THRESHOLD: u32 = 500_000_000;

    const ErrorWrongDeployer: u64 = 1;
    const ErrorAlreadyDeployed: u64 = 2;
    const ErrorWrongFarmTime: u64 = 3;
    const ErrorGainIsZero: u64 = 4;
    const ErrorTotalWeightIsZero: u64 = 5;
    const ErrorDivideZero: u64 = 6;
    const ErrorWrongTotalWeight: u64 = 7;
    const ErrorWrongTimestamp: u64 = 8;
    const ErrorWrongHarvestIndex: u64 = 9;
    const ErrorNotAlive: u64 = 10;
    const ErrorAlreadyStaked: u64 = 11;
    const ErrorNotStaked: u64 = 12;
    const ErrorAssetExist: u64 = 13;
    const ErrorBitcoinClientError: u64 = 14;

    spec module {
        pragma verify = false;
    }


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
            mantissa: num * EXP_SCALE
        }
    }


    fun mantissa(a: Exp): u128 {
        a.mantissa
    }


    fun exp(num: u128, denom: u128): Exp {
        // if overflow move will abort
        let scaledNumerator = num * EXP_SCALE;
        let rational = div_u128(scaledNumerator, denom);
        Exp {
            mantissa: rational
        }
    }

    fun div_u128(a: u128, b: u128): u128 {
        assert!(b != 0, ErrorDivideZero);
        if (a == 0) {
            return 0
        };
        a / b
    }

    fun truncate(exp: Exp): u128 {
        return exp.mantissa / EXP_SCALE
    }

    /// The `GROW Coin`
    struct GROW has key, store {}

    struct FarmingAsset has key {
        asset_total_weight: u64,
        harvest_index: u128,
        last_update_timestamp: u64,
        // Release count per seconds
        release_per_second: u128,
        // Start time, by seconds, user can operate stake only after this timestamp
        start_time: u64,
        // Farming end time
        end_time: u64,
        // Hold the CoinInfo object
        coin_info: Object<CoinInfo<GROW>>,
        // utxo id ==> address
        stake_table: Table<ObjectID, address>,
        // Representing the pool is alive, false: not alive, true: alive.
        alive: bool,
    }

    struct UserStake has key {
        /// utxo ==> stake
        stake: Table<ObjectID, Stake>
    }

    /// To store user's asset token
    struct Stake has key, store {
        asset_weight: u64,
        last_harvest_index: u128,
        gain: u128,
    }

    /// The stake info of UTXO
    /// This Info store in the temporary state area of UTXO
    /// If the UTXO is spent, the stake info will be removed
    struct StakeInfo has store, drop {}


    struct StakeEvent has copy, drop {
        asset_id: ObjectID,
        asset_weight: u64,
        account: address,
        timestamp: u64
    }

    struct UnStakeEvent has copy, drop {
        asset_id: ObjectID,
        asset_weight: u64,
        gain: u128,
        account: address,
        timestamp: u64
    }

    struct HarvestEvent has copy, drop {
        asset_id: ObjectID,
        harvest_index: u128,
        gain: u128,
        account: address,
        timestamp: u64
    }

    struct RemoveExpiredEvent has copy, drop {
        asset_id: ObjectID,
        account: address,
    }

    struct SubscriberInfo has key {
        subscriber: Object<Subscriber<TempStateDropEvent>>,
    }

    fun init() {
        let state_info_name = type_info::type_name<StakeInfo>();
        let subscriber = event_queue::subscribe<TempStateDropEvent>(state_info_name);
        let grow_bitcoin_signer = signer::module_signer<StakeInfo>();
        account::move_resource_to(&grow_bitcoin_signer, SubscriberInfo {
            subscriber
        });
    }


    public entry fun deploy(
        signer: &signer,
        release_per_second: u128,
        start_time: u64,
        end_time: u64,
        name: vector<u8>,
        symbol: vector<u8>,
        decimals: u8,
        _cap: &mut Object<AdminCap>
    ) {
        assert!(signer::address_of(signer) == DEPLOYER, ErrorWrongDeployer);
        assert!(!account::exists_resource<FarmingAsset>(DEPLOYER), ErrorAlreadyDeployed);

        let now_seconds = timestamp::now_seconds();
        let coin_info = coin::register_extend<GROW>(
            string::utf8(name),
            string::utf8(symbol),
            option::none(),
            decimals,
        );
        account::move_resource_to(signer, FarmingAsset {
            asset_total_weight: 0,
            harvest_index: 0,
            last_update_timestamp: now_seconds,
            release_per_second,
            start_time,
            end_time,
            coin_info,
            stake_table: table::new(),
            alive: true
        });
    }

    public fun modify_parameter(
        _cap: &mut Object<AdminCap>,
        release_per_second: u128,
        alive: bool
    ) {
        // Not support to shuttingdown alive state.
        assert!(alive, ErrorNotAlive);

        let farming_asset = account::borrow_mut_resource<FarmingAsset>(DEPLOYER);
        // assert!(farming_asset.alive != alive, Errors::invalid_state(ERR_FARMING_ALIVE_STATE_INVALID));

        let now_seconds = timestamp::now_seconds();

        farming_asset.release_per_second = release_per_second;
        farming_asset.last_update_timestamp = now_seconds;

        // if the pool is alive, then update index
        if (farming_asset.alive) {
            farming_asset.harvest_index = calculate_harvest_index_with_asset(
                farming_asset,
                now_seconds
            );
        };
        farming_asset.alive = alive;
    }

    /// Call by stake user, staking amount of asset in order to get yield farming token
    public entry fun stake(
        signer: &signer,
        asset: &mut Object<UTXO>,
    ) {
        assert!(!utxo::contains_temp_state<StakeInfo>(asset), ErrorAlreadyStaked);
        utxo::add_temp_state(asset, StakeInfo {});
        let asset_weight = value( object::borrow(asset)) * calculate_time_lock_weight(0);
        do_stake(signer, object::id(asset), asset_weight);
    }

    public entry fun stake_bbn(
        signer: &signer,
        asset: &mut Object<BBNStakeSeal>,
    ) {
        assert!(!bbn::contains_temp_state<StakeInfo>(asset), ErrorAlreadyStaked);
        bbn::add_temp_state(asset, StakeInfo {});
        let bbn_stake_seal = object::borrow(asset);
        let asset_weight = bbn::staking_value(bbn_stake_seal) * calculate_time_lock_weight(
            (((bbn::staking_time(bbn_stake_seal) as u64) + bbn::block_height(bbn_stake_seal)) as u32)
        );
        do_stake(signer, object::id(asset), asset_weight);
    }

    fun do_stake(
        signer: &signer,
        asset_id: ObjectID,
        asset_weight: u64
    ) {
        process_expired_state();
        let account = signer::address_of(signer);

        let farming_asset = account::borrow_mut_resource<FarmingAsset>(DEPLOYER);
        assert!(farming_asset.alive, ErrorNotAlive);

        // Check locking time
        let now_seconds = timestamp::now_seconds();
        emit(StakeEvent{
            asset_id,
            asset_weight,
            account,
            timestamp: now_seconds
        });
        assert!(farming_asset.start_time <= now_seconds, ErrorWrongFarmTime);
        assert!(farming_asset.end_time > now_seconds, ErrorWrongFarmTime);

        let time_period = now_seconds - farming_asset.last_update_timestamp;

        if (farming_asset.asset_total_weight <= 0) {
            // Stake as first user
            let gain = farming_asset.release_per_second * (time_period as u128);
            if (!account::exists_resource<UserStake>(account)) {
                account::move_resource_to(signer, UserStake {
                    stake: table::new()
                });
            };
            let stake_table = account::borrow_mut_resource<UserStake>(account);
            table::add(&mut stake_table.stake, asset_id, Stake {
                asset_weight,
                last_harvest_index: 0,
                gain,
            });
            farming_asset.harvest_index = 0;
            farming_asset.asset_total_weight = asset_weight;
        } else {
            let new_harvest_index = calculate_harvest_index_with_asset(farming_asset, now_seconds);
            if (!account::exists_resource<UserStake>(account)) {
                account::move_resource_to(signer, UserStake {
                    stake: table::new()
                });
            };
            let stake_table = account::borrow_mut_resource<UserStake>(account);
            table::add(&mut stake_table.stake, asset_id, Stake {
                asset_weight,
                last_harvest_index: new_harvest_index,
                gain: 0,
            });
            farming_asset.asset_total_weight = farming_asset.asset_total_weight + asset_weight;
            farming_asset.harvest_index = new_harvest_index;
        };
        farming_asset.last_update_timestamp = now_seconds;
        table::add(&mut farming_asset.stake_table, asset_id, account);
    }

    /// Unstake asset from farming pool
    public entry fun unstake(signer: &signer, asset: &mut Object<UTXO>) {
        let coin = do_unstake(signer, object::id(asset));
        utxo::remove_temp_state<StakeInfo>(asset);
        account_coin_store::deposit(sender(), coin);
    }

    public entry fun unstake_bbn(signer: &signer, asset: &mut Object<BBNStakeSeal>) {
        // TODO check bbn stake seal is expired
        let coin = do_unstake(signer, object::id(asset));
        bbn::remove_temp_state<StakeInfo>(asset);
        account_coin_store::deposit(sender(), coin);
    }

    fun do_unstake(signer: &signer, asset_id: ObjectID): Coin<GROW> {
        process_expired_state();
        let account = signer::address_of(signer);
        let farming_asset = account::borrow_mut_resource<FarmingAsset>(DEPLOYER);
        let user_stake = account::borrow_mut_resource<UserStake>(account);
        table::remove(&mut farming_asset.stake_table, asset_id);
        let Stake { last_harvest_index, asset_weight, gain } =
            table::remove(&mut user_stake.stake, asset_id);
        let now_seconds = if (timestamp::now_seconds() < farming_asset.end_time) {
            timestamp::now_seconds()
        } else {
            farming_asset.end_time
        };
        let new_harvest_index = calculate_harvest_index_with_asset(farming_asset, now_seconds);

        let period_gain = calculate_withdraw_amount(new_harvest_index, last_harvest_index, asset_weight);
        let total_gain = gain + period_gain;
        let withdraw_token = coin::mint_extend(&mut farming_asset.coin_info, (total_gain as u256));
        emit(UnStakeEvent{
            asset_id,
            asset_weight,
            gain: total_gain,
            account,
            timestamp: now_seconds
        });
        assert!(farming_asset.asset_total_weight >= asset_weight, ErrorWrongTotalWeight);

        // Update farm asset
        farming_asset.asset_total_weight = farming_asset.asset_total_weight - asset_weight;
        farming_asset.last_update_timestamp = now_seconds;

        if (farming_asset.alive) {
            farming_asset.harvest_index = new_harvest_index;
        };
        withdraw_token
    }

    /// Harvest yield farming token from stake
    public entry fun harvest(signer:&signer, asset: &mut Object<UTXO>) {
        let coin = do_harvest(signer, object::id(asset));
        account_coin_store::deposit(sender(), coin);
    }

    public entry fun harvest_bbn(signer:&signer, asset: &mut Object<BBNStakeSeal>) {
        // TODO check bbn stake seal is expired
        let coin = do_harvest(signer, object::id(asset));
        account_coin_store::deposit(sender(), coin);
    }

    fun do_harvest(signer:&signer, asset_id: ObjectID): Coin<GROW> {
        process_expired_state();
        let farming_asset = account::borrow_mut_resource<FarmingAsset>(DEPLOYER);
        let account = signer::address_of(signer);
        let stake_table = account::borrow_mut_resource<UserStake>(account);
        let stake = table::borrow_mut(&mut stake_table.stake, asset_id);
        let now_seconds = if (timestamp::now_seconds() < farming_asset.end_time) {
            timestamp::now_seconds()
        } else {
            farming_asset.end_time
        };
        let new_harvest_index = calculate_harvest_index_with_asset(farming_asset, now_seconds);

        let period_gain = calculate_withdraw_amount(
            new_harvest_index,
            stake.last_harvest_index,
            stake.asset_weight
        );

        let total_gain = stake.gain + period_gain;
        assert!(total_gain > 0, ErrorGainIsZero);
        // let withdraw_amount = total_gain;
        emit(HarvestEvent{
            asset_id,
            harvest_index: new_harvest_index,
            gain: total_gain,
            account,
            timestamp: now_seconds
        });
        let withdraw_token = coin::mint_extend(&mut farming_asset.coin_info, (total_gain as u256));
        stake.gain = 0;
        stake.last_harvest_index = new_harvest_index;

        if (farming_asset.alive) {
            farming_asset.harvest_index = new_harvest_index;
        };
        farming_asset.last_update_timestamp = now_seconds;

        withdraw_token
    }

    /// The user can quering all yield farming amount in any time and scene
    public fun query_gov_token_amount(
        asset_id: ObjectID
    ): u128 {

        let farming_asset = account::borrow_resource<FarmingAsset>(DEPLOYER);
        if (!table::contains(&farming_asset.stake_table, asset_id)) {
            return 0
        };
        let account = table::borrow(&farming_asset.stake_table, asset_id);

        let stake_table = account::borrow_resource<UserStake>(*account);
        let stake = table::borrow(&stake_table.stake, asset_id);
        let now_seconds = if (timestamp::now_seconds() < farming_asset.end_time) {
            timestamp::now_seconds()
        } else {
            farming_asset.end_time
        };
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
    public fun query_total_stake(): u64 {
        let farming_asset = account::borrow_resource<FarmingAsset>(DEPLOYER);
        farming_asset.asset_total_weight
    }

    /// Query stake weight from user staking objects.
    public fun query_stake(asset_id: ObjectID): u64 {
        let farming_asset = account::borrow_resource<FarmingAsset>(DEPLOYER);
        let account = table::borrow(&farming_asset.stake_table, asset_id);
        let stake_table = account::borrow_resource<UserStake>(*account);
        let stake = table::borrow(&stake_table.stake, asset_id);
        stake.asset_weight
    }

    /// Queyry pool info from pool type
    /// return value: (alive, release_per_second, asset_total_weight, harvest_index)
    public fun query_info(): (bool, u128, u64, u128) {
        let asset = account::borrow_resource<FarmingAsset>(DEPLOYER);
        (
            asset.alive,
            asset.release_per_second,
            asset.asset_total_weight,
            asset.harvest_index
        )
    }

    /// Update farming asset
    fun calculate_harvest_index_with_asset(
        farming_asset: &FarmingAsset,
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
        assert!(last_update_timestamp <= now_seconds, ErrorWrongTimestamp);
        let time_period = now_seconds - last_update_timestamp;
        let addtion_index = release_per_second * ((time_period as u128));
        harvest_index + mantissa(exp_direct_expand(addtion_index))
    }

    /// There is calculating from harvest index and global parameters
    public fun calculate_harvest_index(
        harvest_index: u128,
        asset_total_weight: u64,
        last_update_timestamp: u64,
        now_seconds: u64,
        release_per_second: u128
    ): u128 {
        assert!(asset_total_weight > 0, ErrorTotalWeightIsZero);
        assert!(last_update_timestamp <= now_seconds, ErrorWrongTimestamp);

        let time_period = now_seconds - last_update_timestamp;
        let numr = (release_per_second * (time_period as u128));
        let denom = (asset_total_weight as u128);
        harvest_index + mantissa(exp(numr, denom))
    }

    /// This function will return a gain index
    public fun calculate_withdraw_amount(
        harvest_index: u128,
        last_harvest_index: u128,
        asset_weight: u64
    ): u128 {
        assert!(harvest_index >= last_harvest_index, ErrorWrongHarvestIndex);
        let amount = (asset_weight as u128) * (harvest_index - last_harvest_index);
        truncate(exp_direct(amount))
    }


    /// Check the Farming of AsssetT is exists.
    public fun exists_asset_at(): bool {
        account::exists_resource<FarmingAsset>(DEPLOYER)
    }

    /// Check stake at address exists.
    public fun exists_stake_at_address(account: address): bool {
        account::exists_resource<UserStake>(account)
    }

    public fun check_asset_is_staked(asset_id: ObjectID): (bool, u128) {
        let farming_asset = account::borrow_resource<FarmingAsset>(DEPLOYER);
        if (table::contains(&farming_asset.stake_table, asset_id)) {
            let token_amount = query_gov_token_amount(asset_id);
            return (true, token_amount)
        };
        return (false, 0)
    }

    public fun process_expired_state(){
        let subscriber_info = account::borrow_mut_resource<SubscriberInfo>(@grow_bitcoin);
        let event = event_queue::consume(&mut subscriber_info.subscriber);
        if (option::is_some(&event)){
            let event = option::destroy_some(event);
            let (asset_id, _outpoint, _value) = utxo::unpack_temp_state_drop_event(event);
            remove_expired_stake(asset_id);
        }
    }

    // y = x^(1/2)  x is lock day
    // x not exceeding 1000, so y never over 31 and bbn stake weight is 22
    public fun calculate_time_lock_weight(tx_lock_time: u32): u64{
        if (tx_lock_time < LOCK_TIME_THRESHOLD) {
            // lock_time is a block heigh
            // We assume that each block takes 10 minutes, 1day ~ 144 block
            let btc_block = latest_block_height();
            if (btc_block >= (tx_lock_time as u64)){
                return 1
            };
            let tx_lock_day = ((tx_lock_time as u64) - btc_block)/PerDayBlocks;
            return 1 + u64::sqrt(u64::min(tx_lock_day, MaxLockDay))
        };
        // lock_time is a bitcoin time
        let btc_time = bitcoin::get_bitcoin_time();
        if (btc_time >= tx_lock_time){
            return 1
        };
        let tx_lock_day = (tx_lock_time - btc_time)/PerDaySeconds;

        1 + u64::sqrt(u64::min((tx_lock_day as u64), MaxLockDay))

    }

    fun latest_block_height(): u64 {
        let height_hash = bitcoin::get_latest_block();
        assert!(option::is_some(&height_hash), ErrorBitcoinClientError);
        let (height,_hash) = types::unpack_block_height_hash(option::destroy_some(height_hash));
        height
    }
    
    public entry fun remove_expired_stake(asset_id: ObjectID) {
        let (is_staked, _) = check_asset_is_staked(asset_id);
        assert!(is_staked, ErrorNotStaked);
        assert!((!object::exists_object_with_type<UTXO>(asset_id) && !object::exists_object_with_type<BBNStakeSeal>(asset_id)), ErrorAssetExist);
        let farming_asset = account::borrow_mut_resource<FarmingAsset>(DEPLOYER);
        let account = table::remove(&mut farming_asset.stake_table, asset_id);
        let user_stake = account::borrow_mut_resource<UserStake>(account);
        let Stake {
            asset_weight,
            last_harvest_index: _,
            gain: _ } = table::remove(&mut user_stake.stake, asset_id);
        emit(RemoveExpiredEvent{
            asset_id,
            account
        });
        farming_asset.asset_total_weight = farming_asset.asset_total_weight - asset_weight;
    }

    #[test(sender=@0x42)]
    fun test_stake(sender: signer) {
        bitcoin_move::genesis::init_for_test();
        add_latest_block(100, @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21);
        init();
        let admin_cap_id = object::named_object_id<AdminCap>();
        let grow_bitcoin_signer = signer::module_signer<AdminCap>();
        let admin_cap = object::borrow_mut_object<AdminCap>(&grow_bitcoin_signer, admin_cap_id);
        deploy(&sender, 1, 0, 200, b"BTC Holder Coin", b"HDC", 6, admin_cap);
        let seconds = 100;
        let tx_id = @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21;
        let sat_value = 100000000;
        let utxo = utxo::new_for_testing(tx_id, 0u32, sat_value);
        let utxo2 = utxo::new_for_testing(tx_id, 1u32, sat_value);
        let utxo_id = object::id(&utxo);
        let utxo_id2 = object::id(&utxo2);
        stake(&sender, &mut utxo);
        timestamp::fast_forward_seconds_for_test(seconds);
        stake(&sender, &mut utxo2);
        timestamp::fast_forward_seconds_for_test(seconds);
        let amount = query_gov_token_amount(utxo_id);
        let amount2 = query_gov_token_amount(utxo_id2);
        assert!(amount == 150, 1);
        assert!(amount2 == 50, 2);
        let coin = do_unstake(&sender, object::id(&utxo));
        assert!(coin::value(&coin) == 150, 3);
        let amount = query_gov_token_amount(utxo_id);
        assert!(amount == 0, 4);
        let coin2 = do_harvest(&sender, object::id(&utxo2));
        assert!(coin::value(&coin2) == 50, 4);
        coin::destroy_for_testing(coin2);
        utxo::drop_for_testing(utxo2);
        // remove_expired_stake(utxo_id2);
        utxo::drop_for_testing(utxo);
        coin::destroy_for_testing(coin);
    }

    #[test]
    fun test_calculate_time_lock_weight() {
        bitcoin_move::genesis::init_for_test();
        add_latest_block(100, @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21);
        let weight= calculate_time_lock_weight(64000+100);
        add_latest_block(40000, @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21);
        let weight2= calculate_time_lock_weight(64000+100);
        assert!(weight == 22, 0);
        assert!(weight2 == 13, 0)
    }
}