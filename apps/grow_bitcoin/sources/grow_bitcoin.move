// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module grow_bitcoin::grow_bitcoin {

    use std::string::{Self, String};
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

    const TOTAL_GROW_SUPPLY: u128 = 2100_0000_0000;

    const GROW_NAME:vector<u8> = b"Grow Bitcoin";
    const GROW_SYMBOL:vector<u8> = b"GROW";
    const GROW_ICON_URL: vector<u8> = b"<svg xmlns=\"http://www.w3.org/2000/svg\" id=\"uuid-22caa40a-9fd8-491f-8f74-64b2050f5896\" data-name=\"layer1\" viewBox=\"0 0 317 317\"><defs><style>.uuid-28301967-1909-4b30-9889-78c9dbdaacd3{stroke:#fff;stroke-miterlimit:10}</style></defs><circle cx=\"158.5\" cy=\"158.5\" r=\"158.5\" class=\"uuid-28301967-1909-4b30-9889-78c9dbdaacd3\"/><circle cx=\"158.5\" cy=\"158.5\" r=\"141.25\" class=\"uuid-28301967-1909-4b30-9889-78c9dbdaacd3\"/><path d=\"M79.06 153.89c12.61-50.21 53.37-78.37 93.71-78.37 24.37 0 38.45 10.93 47.06 21.64l-23.95 21.43c-5.67-6.93-12.61-10.92-24.16-10.92-24.58 0-47.48 19.75-55.47 52.11-7.35 29.2 1.05 45.59 28.57 45.59 7.14 0 13.87-2.52 17.86-5.04l7.14-24.16h-24.79l7.78-30.68h57.99l-17.44 74.38c-13.45 10.5-34.25 17.86-54.63 17.86-44.96 0-73.12-29-59.67-83.83Z\" style=\"stroke-miterlimit:10;fill:#fff;stroke-width:.5px;stroke:#ff9908\"/><path d=\"M221.86 209.96v-4.23c0-3.97-.07-5.43-.22-8.31l-4.42-.65v-2.84l14.24-4.8 1.24.77.66 8.87v11.19c0 4.48.07 11.91.21 15.16h-11.94c.14-3.25.22-10.69.22-15.16Zm-4.06 11.8 6.9-1.45h7.59l6.9 1.45v3.36H217.8zm10.23-21.34h5.42l-.97 1.96c1.27-8.85 6.3-13.26 10.61-13.26 3.13 0 5.67 1.74 6.27 5.32-.16 3.76-2.24 5.89-5.13 5.89-2.18 0-3.86-1.07-5.58-3.35l-2.11-2.75 2.92 1.05c-2.6 1.89-5.2 5.71-6.13 10.02l-5.3-.52v-4.37Z\" style=\"stroke-miterlimit:10;stroke:#ff9908;fill:#ff9908\"/></svg>";

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
    const ErrorWrongTimeRange: u64 = 15;


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
        asset_total_value: u64,
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
        asset_type: String,
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
        asset_type: String,
        asset_weight: u64,
        account: address,
        timestamp: u64
    }

    struct UnStakeEvent has copy, drop {
        asset_id: ObjectID,
        asset_type: String,
        asset_weight: u64,
        gain: u128,
        account: address,
        timestamp: u64
    }

    struct HarvestEvent has copy, drop {
        asset_id: ObjectID,
        asset_type: String,
        harvest_index: u128,
        gain: u128,
        account: address,
        timestamp: u64
    }

    struct RemoveExpiredEvent has copy, drop {
        asset_id: ObjectID,
        asset_type: String,
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

        let start_time = timestamp::now_seconds();
        let duration_seconds = (PerDaySeconds as u64) *180u64;
        let end_time = start_time + duration_seconds;
        let release_per_second = TOTAL_GROW_SUPPLY / (duration_seconds as u128);
        do_deploy(release_per_second, start_time, end_time);
    }


    fun do_deploy(
        release_per_second: u128,
        start_time: u64,
        end_time: u64,
    ) {
        assert!(!account::exists_resource<FarmingAsset>(DEPLOYER), ErrorAlreadyDeployed);
        assert!(start_time < end_time, ErrorWrongTimeRange);

        let now_seconds = timestamp::now_seconds();
        assert!(start_time >= now_seconds, ErrorWrongTimeRange);

        let coin_info = coin::register_extend<GROW>(
            string::utf8(GROW_NAME),
            string::utf8(GROW_SYMBOL),
            option::some(string::utf8(GROW_ICON_URL)),
            0u8,
        );
        let grow_bitcoin_signer = signer::module_signer<GROW>();
        account::move_resource_to(&grow_bitcoin_signer, FarmingAsset {
            asset_total_value: 0,
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

    public entry fun update_coin_icon(){
        let farming_asset = account::borrow_mut_resource<FarmingAsset>(DEPLOYER);
        coin::upsert_icon_url(&mut farming_asset.coin_info, string::utf8(GROW_ICON_URL));
    }

    public fun modify_parameter(
        release_per_second: u128,
        alive: bool,
        _cap: &mut Object<AdminCap>,
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
        let utxo_value = value( object::borrow(asset));
        let asset_weight = utxo_value * calculate_time_lock_weight(0);
        do_stake(signer, asset, utxo_value, asset_weight);
    }

    public entry fun stake_bbn(
        signer: &signer,
        asset: &mut Object<BBNStakeSeal>,
    ) {
        assert!(!bbn::contains_temp_state<StakeInfo>(asset), ErrorAlreadyStaked);
        bbn::add_temp_state(asset, StakeInfo {});
        let bbn_stake_seal = object::borrow(asset);
        let stake_value = bbn::staking_value(bbn_stake_seal);
        let asset_weight = stake_value * calculate_time_lock_weight(
            (((bbn::staking_time(bbn_stake_seal) as u64) + bbn::block_height(bbn_stake_seal)) as u32)
        );
        do_stake(signer, asset, stake_value, asset_weight);
    }

    fun do_stake<T: key>(
        signer: &signer,
        asset: &mut Object<T>,
        asset_value: u64,
        asset_weight: u64
    ) {
        let asset_id = object::id(asset);
        let asset_type = type_info::type_name<T>();

        process_expired_state();
        let account = signer::address_of(signer);

        let farming_asset = account::borrow_mut_resource<FarmingAsset>(DEPLOYER);
        assert!(farming_asset.alive, ErrorNotAlive);

        // Check locking time
        let now_seconds = timestamp::now_seconds();
        emit(StakeEvent{
            asset_id,
            asset_type,
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
                asset_type,
                asset_weight,
                last_harvest_index: 0,
                gain,
            });
            farming_asset.harvest_index = 0;
            farming_asset.asset_total_value = asset_value;
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
                asset_type,
                asset_weight,
                last_harvest_index: new_harvest_index,
                gain: 0,
            });
            farming_asset.asset_total_value = farming_asset.asset_total_value + asset_value;
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
        let Stake { asset_type, last_harvest_index, asset_weight, gain } =
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
            asset_type,
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
            asset_type: stake.asset_type,
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

    /// Query total stake value(in satoshis) and weight from yield farming resource
    public fun query_total_stake(): (u64, u64) {
        let farming_asset = account::borrow_resource<FarmingAsset>(DEPLOYER);
        (farming_asset.asset_total_value, farming_asset.asset_total_weight)
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
        if (account::exists_resource<UserStake>(account)) {
            let stake = account::borrow_resource<UserStake>(account);
            return !table::is_empty(&stake.stake)
        };
        return false
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
        if (!is_staked || object::exists_object_with_type<UTXO>(asset_id) || object::exists_object_with_type<BBNStakeSeal>(asset_id)){
            return
        };
        let farming_asset = account::borrow_mut_resource<FarmingAsset>(DEPLOYER);
        let account = table::remove(&mut farming_asset.stake_table, asset_id);
        let user_stake = account::borrow_mut_resource<UserStake>(account);
        let Stake {
            asset_type,
            asset_weight,
            last_harvest_index: _,
            gain: _ } = table::remove(&mut user_stake.stake, asset_id);
        emit(RemoveExpiredEvent{
            asset_id,
            asset_type,
            account,
        });
        farming_asset.asset_total_weight = farming_asset.asset_total_weight - asset_weight;
    }

    #[test(sender=@0x42)]
    fun test_stake(sender: signer) {
        bitcoin_move::genesis::init_for_test();
        add_latest_block(100, @0x77dfc2fe598419b00641c296181a96cf16943697f573480b023b77cce82ada21);
        init();
        let _admin_cap = app_admin::admin::init_for_test();
        //deploy(1, 0, 200, admin_cap);
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
        let (total_value, total_weight) = query_total_stake();
        assert!(total_value == 2 * sat_value, 1);
        assert!(total_weight == 2 * sat_value * calculate_time_lock_weight(0), 2);
        timestamp::fast_forward_seconds_for_test(seconds);
        let amount = query_gov_token_amount(utxo_id);
        let amount2 = query_gov_token_amount(utxo_id2);
        // std::debug::print(&amount);
        // std::debug::print(&amount2);
        assert!(amount == 2025450, 1);
        assert!(amount2 == 675150, 2);
        let coin = do_unstake(&sender, object::id(&utxo));
        assert!(coin::value(&coin) == (amount as u256), 3);
        let amount = query_gov_token_amount(utxo_id);
        assert!(amount == 0, 4);
        let coin2 = do_harvest(&sender, object::id(&utxo2));
        assert!(coin::value(&coin2) == (amount2 as u256), 5);
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