// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::rooch_examples {
    use std::bcs;
    use std::hash;
    use std::option::{Self, Option};
    use std::signer;
    use std::string::{Self, String};
    use std::vector;

    use moveos_std::event::Self;
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::context::{Self, Context};
    use rooch_framework::coin;
    use rooch_framework::account_coin_store;
    use rooch_examples::timestamp;
    use rooch_framework::account::{Self, SignerCapability};

    #[test_only]
    use rooch_framework::genesis;

    const SEED: vector<u8> = b"SEED";
    const EXPIRATION_TIME_IN_SECONDS: u64 = 60 * 60;

    const DECISION_NOT_MADE: u64 = 0;
    const DECISION_SPLIT: u64 = 1;
    const DECISION_STEAL: u64 = 2;

    const ErrorStateIsNotInitialized: u64 = 0;
    const ErrorSignerIsNotDeployer: u64 = 1;
    const ErrorSignerHasInsufficientAptBalance: u64 = 2;
    const ErrorGameDoesNotExist: u64 = 3;
    const ErrorPlayerDoesNotParticipateInTheGame: u64 = 4;
    const ErrorIncorrectHashValue: u64 = 5;
    const ErrorGameNotExpiredYet: u64 = 6;
    const ErrorBothPlayersDoNotHaveDecisionsSubmitted: u64 = 7;
    const ErrorPlayerHasDecisionSubmitted: u64 = 8;

    struct ResouceAccountAddress has key {
        addr: address
    }

    struct State has key {
        next_game_id: u128,
        games: SimpleMap<u128, Game>,
        cap: SignerCapability,
    }

    struct WGBCOIN has key, store {}

    struct Game has store, copy, drop {
        prize_pool_amount: u256,
        player_one: PlayerData,
        player_two: PlayerData,
        expiration_timestamp_in_seconds: u64,
    }

    struct PlayerData has store, copy, drop {
        player_address: address,
        decision_hash: Option<vector<u8>>,
        salt_hash: Option<vector<u8>>,
        decision: u64
    }

    struct CreateGameEvent has store, drop {
        game_id: u128,
        prize_pool_amount: u256,
        player_one_address: address,
        player_two_address: address,
        expiration_timestamp_in_seconds: u64,
        event_creation_timestamp_in_seconds: u64
    }


    struct SubmitDecisionEvent has store, drop {
        game_id: u128,
        player_address: address,
        decision_hash: vector<u8>,
        salt_hash: vector<u8>,
        event_creation_timestamp_in_seconds: u64
    }

    struct RevealDecisionEvent has store, drop {
        game_id: u128,
        player_address: address,
        decision: u64,
        event_creation_timestamp_in_seconds: u64
    }

    struct ConcludeGameEvent has store, drop {
        game_id: u128,
        player_one_decision: u64,
        player_two_decision: u64,
        prize_pool_amount: u256,
        event_creation_timestamp_in_seconds: u64
    }

    struct ReleaseFundsAfterExpirationEvent has store, drop {
        game_id: u128,
        player_one_decision: u64,
        player_two_decision: u64,
        prize_pool_amount: u256,
        event_creation_timestamp_in_seconds: u64
    }

    fun init(account: &signer, ctx: &mut Context) {
        // let source_addr = signer::address_of(account);
        let (signer, cap) = account::create_resource_account(ctx, account);
        let resource_address = signer::address_of(&signer);

        context::move_resource_to(ctx, account, ResouceAccountAddress {
            addr: resource_address
        });
        
        coin::register_extend<WGBCOIN>(ctx,string::utf8(b"WGBCOIN"),string::utf8(b"WGB"), 8);

        let coin = coin::mint_extend<WGBCOIN>(ctx, 1000 * 1000 * 1000);
        account_coin_store::do_accept_coin<WGBCOIN>(ctx,account);
        account_coin_store::do_accept_coin<WGBCOIN>(ctx,&signer);
        account_coin_store::deposit_extend(ctx, signer::address_of(account), coin);

        context::move_resource_to(ctx, &signer, State {
            next_game_id: 0,
            games: simple_map::create(),
            cap
        });
    }

    public entry fun create_game(
        account: &signer,
        prize_pool_amount: u256,
        player_one_address: address,
        player_two_address: address,
        ctx: &mut Context
    ) {
        check_if_state_exists(ctx);
        let now = timestamp::now_seconds(ctx);
        check_if_signer_is_contract_deployer(account);
        let resouce_address = context::borrow_resource<ResouceAccountAddress>(ctx, @rooch_examples).addr;
        let next_game_id = {
            let state_mut_ref = context::borrow_mut_resource<State>(ctx, resouce_address);
            get_next_game_id(&mut state_mut_ref.next_game_id)
        };

        let new_game = Game {
            prize_pool_amount,
            player_one: PlayerData {
                player_address: player_one_address,
                decision_hash: option::none(),
                salt_hash: option::none(),
                decision: DECISION_NOT_MADE
            },
            player_two: PlayerData {
                player_address: player_two_address,
                decision_hash: option::none(),
                salt_hash: option::none(),
                decision: DECISION_NOT_MADE
            },
            expiration_timestamp_in_seconds: EXPIRATION_TIME_IN_SECONDS + now,
        };
        {
            let state_mut_ref = context::borrow_mut_resource<State>(ctx, resouce_address);
            simple_map::add(&mut state_mut_ref.games, next_game_id, new_game);
        };
        account_coin_store::transfer<WGBCOIN>(ctx, account, resouce_address, prize_pool_amount);
        event::emit(
            CreateGameEvent {
                game_id: next_game_id,
                prize_pool_amount,
                player_one_address,
                player_two_address,
                expiration_timestamp_in_seconds: EXPIRATION_TIME_IN_SECONDS + now,
                event_creation_timestamp_in_seconds: now
            }
        );
    }

    public entry fun submit_decision(
        player: &signer,
        game_id: u128,
        decision_hash: vector<u8>,
        salt_hash: vector<u8>,
        ctx: &mut Context
    ) {
        check_if_state_exists(ctx);
        let now = timestamp::now_seconds(ctx);
        let resouce_address = context::borrow_resource<ResouceAccountAddress>(ctx, @rooch_examples).addr;
        let state_mut_ref = context::borrow_mut_resource<State>(ctx, resouce_address);
        check_if_game_exists(&state_mut_ref.games, &game_id);
        let game_mut_ref = simple_map::borrow_mut(&mut state_mut_ref.games, &game_id);
        check_if_player_participates_in_the_game(player, game_mut_ref);
        let player_address = signer::address_of(player);
        check_if_player_does_not_have_a_decision_submitted(game_mut_ref, player_address);
        let player_data_mut_ref = if (game_mut_ref.player_one.player_address == player_address) {
            &mut game_mut_ref.player_one
        }
        else {
            &mut game_mut_ref.player_two
        };
        option::fill(&mut player_data_mut_ref.decision_hash, decision_hash);
        option::fill(&mut player_data_mut_ref.salt_hash, salt_hash);

        event::emit(
            SubmitDecisionEvent {
                game_id,
                player_address,
                decision_hash,
                salt_hash,
                event_creation_timestamp_in_seconds: now
            }
        );
    }

    public entry fun reveal_decision(
        player: &signer,
        game_id: u128,
        salt: String,
        ctx: &mut Context
    ) {
        check_if_state_exists(ctx);
        let now = timestamp::now_seconds(ctx);
        let resouce_address = context::borrow_resource<ResouceAccountAddress>(ctx, @rooch_examples).addr;
        let (game_id, player_address, decision) = {
            let state_mut_ref = context::borrow_mut_resource<State>(ctx, resouce_address);


            check_if_game_exists(&state_mut_ref.games, &game_id);
            let game_mut_ref = simple_map::borrow_mut(&mut state_mut_ref.games, &game_id);
            check_if_player_participates_in_the_game(player, game_mut_ref);
            check_if_both_players_have_a_decision_submitted(game_mut_ref);
            let player_address = signer::address_of(player);
            let (current_player_data_mut_ref, another_player_data_mut_ref) = if (game_mut_ref.player_one.player_address == player_address) {
                (&mut game_mut_ref.player_one, &mut game_mut_ref.player_two)
            }
            else {
                (&mut game_mut_ref.player_two, &mut game_mut_ref.player_one)
            };
            let decision = make_decision(current_player_data_mut_ref, &salt);

            if (another_player_data_mut_ref.decision != DECISION_NOT_MADE) {
                let resouce_account_signer = &account::create_signer_with_capability(&state_mut_ref.cap);

                let (_, game) = simple_map::remove(&mut state_mut_ref.games, &game_id);
                if ((game.player_one.decision == game.player_two.decision) && (game.player_one.decision == DECISION_SPLIT)) {
                    let player_one_amount = game.prize_pool_amount / 2;
                    let player_two_amount = game.prize_pool_amount - player_one_amount;
                    account_coin_store::transfer<WGBCOIN>(
                        ctx,
                        resouce_account_signer,
                        game.player_one.player_address,
                        player_one_amount
                    );
                    account_coin_store::transfer<WGBCOIN>(
                        ctx,
                        resouce_account_signer,
                        game.player_two.player_address,
                        player_two_amount
                    );
                }else if ((game.player_one.decision != game.player_two.decision)) {
                    let steal_player_address = if (game.player_one.decision == DECISION_SPLIT) {
                        game.player_two.player_address
                    }else {
                        game.player_one.player_address
                    };
                    account_coin_store::transfer<WGBCOIN>(ctx, resouce_account_signer, steal_player_address, game.prize_pool_amount);
                }else {
                    account_coin_store::transfer<WGBCOIN>(ctx, resouce_account_signer, @rooch_examples, game.prize_pool_amount);
                };
                event::emit(
                    ConcludeGameEvent {
                        game_id,
                        player_one_decision: game.player_one.decision,
                        player_two_decision: game.player_two.decision,
                        prize_pool_amount: game.prize_pool_amount,
                        event_creation_timestamp_in_seconds: now
                    }
                );
            };
            (game_id, player_address, decision)
        };
        {
            event::emit(
                RevealDecisionEvent {
                    game_id,
                    player_address,
                    decision,
                    event_creation_timestamp_in_seconds: now
                }
            );
        };
    }

    public entry fun release_funds_after_expiration(_account: &signer, game_id: u128,
                                                    ctx: &mut Context) {
        check_if_state_exists(ctx);
        let now = timestamp::now_seconds(ctx);
        let resouce_address = context::borrow_resource<ResouceAccountAddress>(ctx, @rooch_examples).addr;
        let (game, resouce_account_signer) = {
            let state_mut_ref = context::borrow_mut_resource<State>(ctx, resouce_address);
            check_if_game_exists(&state_mut_ref.games, &game_id);

            let (_, game) = simple_map::remove(&mut state_mut_ref.games, &game_id);
            let resouce_account_signer = &account::create_signer_with_capability(&state_mut_ref.cap);
            (game, resouce_account_signer)
        };

        check_if_game_expired(&game, ctx);

        if (game.player_one.decision == game.player_two.decision) {
            account_coin_store::transfer<WGBCOIN>(
                ctx, 
                resouce_account_signer, 
                @rooch_examples, 
                game.prize_pool_amount
            );
        }else if (game.player_one.decision != DECISION_NOT_MADE) {
            account_coin_store::transfer<WGBCOIN>(
                ctx,
                resouce_account_signer,
                game.player_one.player_address,
                game.prize_pool_amount
            );
        }else {
            account_coin_store::transfer<WGBCOIN>(
                ctx,
                resouce_account_signer,
                game.player_two.player_address,
                game.prize_pool_amount
            );
        };

        event::emit(
            ReleaseFundsAfterExpirationEvent {
                game_id,
                player_one_decision: game.player_one.decision,
                player_two_decision: game.player_two.decision,
                prize_pool_amount: game.prize_pool_amount,
                event_creation_timestamp_in_seconds: now
            }
        );
    }

    fun make_decision(player_data: &mut PlayerData, salt: &String): u64 {
        let bytes_salt = *string::bytes(salt);
        check_if_hash_is_correct(*option::borrow(&player_data.salt_hash), bytes_salt);
        let split_hash = {
            let split_value = bcs::to_bytes(&DECISION_SPLIT);
            vector::append(&mut split_value, bytes_salt);
            hash::sha3_256(split_value)
        };
        let steal_hash = {
            let steal_value = bcs::to_bytes(&DECISION_STEAL);
            vector::append(&mut steal_value, bytes_salt);
            hash::sha3_256(steal_value)
        };
        let decision_hash = option::borrow(&player_data.decision_hash);
        let decision = if (&split_hash == decision_hash) {
            DECISION_SPLIT
        }else if (&steal_hash == decision_hash) {
            DECISION_STEAL
        }else {
            abort 100
        };
        player_data.decision = decision;
        decision
    }

    fun get_next_game_id(next_game_id: &mut u128): u128 {
        let now_next_game_id = *next_game_id;
        *next_game_id = *next_game_id + 1;
        return now_next_game_id
    }

    fun check_if_state_exists(ctx: &mut Context) {
        assert!(context::exists_resource<ResouceAccountAddress>(ctx, @rooch_examples), ErrorStateIsNotInitialized);
        let resouce_address = context::borrow_resource<ResouceAccountAddress>(ctx, @rooch_examples).addr;
        assert!(context::exists_resource<State>(ctx, resouce_address), ErrorStateIsNotInitialized);
    }

    fun check_if_signer_is_contract_deployer(signer: &signer) {
        assert!(signer::address_of(signer) == @rooch_examples, ErrorSignerIsNotDeployer);
    }

    fun check_if_account_has_enough_apt_coins(account: &signer, amount: u256, ctx: &Context, ) {
        assert!(account_coin_store::balance<WGBCOIN>(ctx, signer::address_of(account)) >= amount, ErrorSignerHasInsufficientAptBalance);
    }

    fun check_if_game_exists(games: &SimpleMap<u128, Game>, game_id: &u128) {
        assert!(simple_map::contains_key(games, game_id), ErrorGameDoesNotExist);
    }

    fun check_if_player_participates_in_the_game(player: &signer, game: &Game) {
        let player_address = signer::address_of(player);
        assert!(
            game.player_two.player_address == player_address || game.player_one.player_address == player_address,
            ErrorPlayerDoesNotParticipateInTheGame
        );
    }

    fun check_if_both_players_have_a_decision_submitted(game: &Game) {
        assert!(
            option::is_some(&game.player_one.decision_hash) && option::is_some(&game.player_two.decision_hash),
            ErrorBothPlayersDoNotHaveDecisionsSubmitted
        );
    }

    fun check_if_player_does_not_have_a_decision_submitted(game: &Game, player_address: address) {
        assert!(
            game.player_two.player_address == player_address || game.player_one.player_address == player_address,
            ErrorPlayerDoesNotParticipateInTheGame
        );
        let player_data_ref = if (game.player_one.player_address == player_address) {
            &game.player_one
        }
        else if (game.player_two.player_address == player_address) {
            &game.player_two
        }
        else {
            abort ErrorPlayerDoesNotParticipateInTheGame
        };

        assert!(option::is_none(&player_data_ref.decision_hash), ErrorPlayerHasDecisionSubmitted);
    }

    fun check_if_hash_is_correct(hash: vector<u8>, value: vector<u8>) {
        assert!(hash::sha3_256(value) == hash, ErrorIncorrectHashValue);
    }

    fun check_if_game_expired(game: &Game, ctx: &Context) {
        assert!(game.expiration_timestamp_in_seconds <= timestamp::now_seconds(ctx), ErrorGameNotExpiredYet);
    }


    #[test]
    fun test_init() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);
        let resouce_address = context::borrow_resource<ResouceAccountAddress>(ctx, @rooch_examples).addr;

        let state = context::borrow_mut_resource<State>(ctx, resouce_address);
        assert!(state.next_game_id == 0, 0);
        assert!(simple_map::length(&state.games) == 0, 1);

        assert!(
            account::create_signer_with_capability(&state.cap) == account::create_signer_for_test(resouce_address),
            13
        );
        assert!(account_coin_store::is_accept_coin<WGBCOIN>(ctx, resouce_address), 12);
        context::drop_test_context(storage_context);
    }

    #[test]
    #[expected_failure(abort_code = 196615, location = rooch_framework::account)]
    fun test_init_again() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;
        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);
        init(account, ctx);
        context::drop_test_context(storage_context);
    }

    #[test]
    fun test_create_game() {
        let storage_context = genesis::init_for_test();
        
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);
 
        init(account, ctx);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;

        
        timestamp::update_global_time_for_test_secs(10, ctx);
        create_game(account, prize_pool_amount, player_one_address, player_two_address, ctx);
        let resouce_address = context::borrow_resource<ResouceAccountAddress>(ctx, @rooch_examples).addr;

        let state = context::borrow_mut_resource<State>(ctx, resouce_address);
        assert!(state.next_game_id == 1, 0);
        assert!(simple_map::length(&state.games) == 1, 1);
        assert!(simple_map::contains_key(&state.games, &0), 2);


        let game = *simple_map::borrow(&state.games, &0);
        assert!(game.prize_pool_amount == prize_pool_amount, 13);
        assert!(game.expiration_timestamp_in_seconds >= 3610 && game.expiration_timestamp_in_seconds <= 3611, 14);
        assert!(game.player_one.player_address == player_one_address, 15);
        assert!(option::is_none(&game.player_one.decision_hash), 16);
        assert!(option::is_none(&game.player_one.salt_hash), 17);
        assert!(game.player_one.decision == DECISION_NOT_MADE, 18);

        assert!(game.player_two.player_address == player_two_address, 19);
        assert!(option::is_none(&game.player_two.decision_hash), 20);
        assert!(option::is_none(&game.player_two.salt_hash), 21);
        assert!(game.player_two.decision == DECISION_NOT_MADE, 22);

        assert!(account_coin_store::balance<WGBCOIN>(ctx, resouce_address) == prize_pool_amount, 23);

        context::drop_test_context(storage_context);
    }


    #[test]
    fun test_submit_decision() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10, ctx);

        let (player_one, _player_two) = {
            (account::create_account_for_test(ctx, player_one_address), account::create_account_for_test(
                ctx,
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address, ctx);


        let resouce_address = context::borrow_resource<ResouceAccountAddress>(ctx, @rooch_examples).addr;

        let salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut decision, salt);

        let decision_hash = hash::sha3_256(decision);
        let salt_hash = hash::sha3_256(salt);

        submit_decision(&player_one, 0, decision_hash, salt_hash, ctx);

        let state = context::borrow_mut_resource<State>(ctx, resouce_address);

        assert!(state.next_game_id == 1, 0);
        assert!(simple_map::length(&state.games) == 1, 1);
        assert!(simple_map::contains_key(&state.games, &0), 2);

        let game = simple_map::borrow(&state.games, &0);
        assert!(game.prize_pool_amount == prize_pool_amount, 13);
        assert!(game.expiration_timestamp_in_seconds >= 3610 && game.expiration_timestamp_in_seconds <= 3611, 14);

        assert!(game.player_one.player_address == player_one_address, 15);
        assert!(option::contains(&game.player_one.decision_hash, &decision_hash), 16);
        assert!(option::contains(&game.player_one.salt_hash, &salt_hash), 17);
        assert!(game.player_one.decision == DECISION_NOT_MADE, 18);

        assert!(game.player_two.player_address == player_two_address, 19);
        assert!(option::is_none(&game.player_two.decision_hash), 20);
        assert!(option::is_none(&game.player_two.salt_hash), 21);
        assert!(game.player_two.decision == DECISION_NOT_MADE, 22);

        assert!(account_coin_store::balance<WGBCOIN>(ctx, resouce_address) == prize_pool_amount, 24);
        context::drop_test_context(storage_context);
    }


    #[test]
    #[expected_failure(abort_code = 8, location = Self)]
    fun test_submit_decision_player_one_has_a_decision_submitted() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10, ctx);

        let (player_one, _player_two) = {
            (account::create_account_for_test(ctx, player_one_address), account::create_account_for_test(
                ctx,
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address, ctx);
        let salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut decision, salt);

        let decision_hash = hash::sha3_256(decision);
        let salt_hash = hash::sha3_256(salt);

        submit_decision(&player_one, 0, decision_hash, salt_hash, ctx);
        submit_decision(&player_one, 0, decision_hash, salt_hash, ctx);
        context::drop_test_context(storage_context);
    }


    #[test]
    fun test_reveal_decision_split() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10, ctx);

        let (player_one, player_two) = {
            (account::create_account_for_test(ctx, player_one_address), account::create_account_for_test(
                ctx,
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address, ctx);

        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_two);
        let player_one_salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut decision, player_one_salt);

        let player_one_decision_hash = hash::sha3_256(decision);
        let player_one_salt_hash = hash::sha3_256(player_one_salt);

        submit_decision(&player_one, 0, player_one_decision_hash, player_one_salt_hash, ctx);


        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash, ctx);

        reveal_decision(&player_one, 0, string::utf8(player_one_salt), ctx);
        {
            let resouce_address = context::borrow_resource<ResouceAccountAddress>(ctx, @rooch_examples).addr;
            {
                let state = context::borrow_mut_resource<State>(ctx, resouce_address);
                assert!(state.next_game_id == 1, 0);
                assert!(simple_map::length(&state.games) == 1, 1);
                assert!(simple_map::contains_key(&state.games, &0), 2);

                let game = simple_map::borrow(&state.games, &0);
                assert!(game.prize_pool_amount == prize_pool_amount, 13);
                assert!(
                    game.expiration_timestamp_in_seconds >= 3610 && game.expiration_timestamp_in_seconds <= 3611,
                    14
                );

                assert!(game.player_one.player_address == player_one_address, 15);
                assert!(option::contains(&game.player_one.decision_hash, &player_one_decision_hash), 16);
                assert!(option::contains(&game.player_one.salt_hash, &player_one_salt_hash), 17);
                assert!(game.player_one.decision == DECISION_SPLIT, 18);
                assert!(game.player_two.player_address == player_two_address, 19);
                assert!(option::contains(&game.player_two.decision_hash, &player_two_decision_hash), 20);
                assert!(option::contains(&game.player_two.salt_hash, &player_two_salt_hash), 21);
                assert!(game.player_two.decision == DECISION_NOT_MADE, 22);
            };

            assert!(account_coin_store::balance<WGBCOIN>(ctx, resouce_address) == prize_pool_amount, 23);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_one_address) == 0, 24);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_two_address) == 0, 25);

            reveal_decision(&player_two, 0, string::utf8(player_two_salt), ctx);
            {
                let state = context::borrow_mut_resource<State>(ctx, resouce_address);
                assert!(state.next_game_id == 1, 28);
                assert!(simple_map::length(&state.games) == 0, 29);
            };

            assert!(account_coin_store::balance<WGBCOIN>(ctx, resouce_address) == 0, 40);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_one_address) == prize_pool_amount / 2, 42);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_two_address) == prize_pool_amount / 2, 43);
            context::drop_test_context(storage_context);
        }
    }

    #[test]
    fun test_reveal_decision_player_one_steals() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10, ctx);

        let (player_one, player_two) = {
            (account::create_account_for_test(ctx, player_one_address), account::create_account_for_test(
                ctx,
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address, ctx);

        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_two);
        let player_one_salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut decision, player_one_salt);

        let player_one_decision_hash = hash::sha3_256(decision);
        let player_one_salt_hash = hash::sha3_256(player_one_salt);

        submit_decision(&player_one, 0, player_one_decision_hash, player_one_salt_hash, ctx);


        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash, ctx);

        reveal_decision(&player_one, 0, string::utf8(player_one_salt), ctx);
        reveal_decision(&player_two, 0, string::utf8(player_two_salt), ctx);
        {
            let resouce_address = context::borrow_resource<ResouceAccountAddress>(
                ctx,
                @rooch_examples
            ).addr;

            {
                let state = context::borrow_mut_resource<State>(ctx, resouce_address);
                assert!(state.next_game_id == 1, 0);
            };
            assert!(account_coin_store::balance<WGBCOIN>(ctx, resouce_address) == 0, 40);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_one_address) == prize_pool_amount, 42);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_two_address) == 0, 43);
        };
        context::drop_test_context(storage_context);
    }

    #[test]
    fun test_reveal_decision_player_two_steals() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10, ctx);

        let (player_one, player_two) = {
            (account::create_account_for_test(ctx, player_one_address), account::create_account_for_test(
                ctx,
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address, ctx);

        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_two);
        let player_one_salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut decision, player_one_salt);

        let player_one_decision_hash = hash::sha3_256(decision);
        let player_one_salt_hash = hash::sha3_256(player_one_salt);

        submit_decision(&player_one, 0, player_one_decision_hash, player_one_salt_hash, ctx);


        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash, ctx);

        reveal_decision(&player_one, 0, string::utf8(player_one_salt), ctx);
        reveal_decision(&player_two, 0, string::utf8(player_two_salt), ctx);


        {
            let resouce_address = context::borrow_resource<ResouceAccountAddress>(
                ctx,
                @rooch_examples
            ).addr;

            {
                let state = context::borrow_mut_resource<State>(ctx, resouce_address);
                assert!(state.next_game_id == 1, 0);
            };
            assert!(account_coin_store::balance<WGBCOIN>(ctx, resouce_address) == 0, 40);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_one_address) == 0, 42);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_two_address) == prize_pool_amount, 43);
        };
        context::drop_test_context(storage_context);
    }

    #[test]
    fun test_reveal_decision_both_players_steal() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10, ctx);

        let (player_one, player_two) = {
            (account::create_account_for_test(ctx, player_one_address), account::create_account_for_test(
                ctx,
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address, ctx);

        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_two);
        let player_one_salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut decision, player_one_salt);

        let player_one_decision_hash = hash::sha3_256(decision);
        let player_one_salt_hash = hash::sha3_256(player_one_salt);

        submit_decision(&player_one, 0, player_one_decision_hash, player_one_salt_hash, ctx);


        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash, ctx);

        reveal_decision(&player_one, 0, string::utf8(player_one_salt), ctx);
        reveal_decision(&player_two, 0, string::utf8(player_two_salt), ctx);

        {
            let resouce_address = context::borrow_resource<ResouceAccountAddress>(
                ctx,
                @rooch_examples
            ).addr;

            {
                let state = context::borrow_mut_resource<State>(ctx, resouce_address);
                assert!(state.next_game_id == 1, 0);
            };
            assert!(account_coin_store::balance<WGBCOIN>(ctx, resouce_address) == 0, 40);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_one_address) == 0, 42);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_two_address) == 0, 43);
        };
        context::drop_test_context(storage_context);
    }


    #[test]
    #[expected_failure(abort_code = 7, location = Self)]
    fun test_reveal_decision_player_one_does_not_have_a_decision_submitted() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10, ctx);

        let (player_one, player_two) = {
            (account::create_account_for_test(ctx, player_one_address), account::create_account_for_test(
                ctx,
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address, ctx);

        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_two);

        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash, ctx);
        reveal_decision(&player_two, 0, string::utf8(player_two_salt), ctx);
        context::drop_test_context(storage_context);
    }

    #[test]
    fun test_release_funds_after_expiration_transfer_to_creator() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10, ctx);

        let (player_one, player_two) = {
            (account::create_account_for_test(ctx, player_one_address), account::create_account_for_test(
                ctx,
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address, ctx);

        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_two);

        timestamp::update_global_time_for_test_secs(3612, ctx);
        release_funds_after_expiration(account, 0, ctx);
        {
            let resouce_address = context::borrow_resource<ResouceAccountAddress>(
                ctx,
                @rooch_examples
            ).addr;

            {
                let state = context::borrow_mut_resource<State>(ctx, resouce_address);
                assert!(state.next_game_id == 1, 0);
            };
            assert!(account_coin_store::balance<WGBCOIN>(ctx, resouce_address) == 0, 13);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_one_address) == 0, 14);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_two_address) == 0, 15);
        };
        context::drop_test_context(storage_context);
    }

    #[test]
    fun test_release_funds_after_expiration_transfer_to_player_one() {
        let storage_context = genesis::init_for_test();
        let ctx = &mut storage_context;

        let account = &account::create_account_for_test(ctx, @rooch_examples);
        let _move_os = &account::create_account_for_test(ctx, @moveos_std);
        timestamp::set_time_has_started_for_testing(ctx);

        init(account, ctx);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10, ctx);

        let (player_one, player_two) = {
            (account::create_account_for_test(ctx, player_one_address), account::create_account_for_test(
                ctx,
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address, ctx);

        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(ctx, &player_two);
        let player_one_salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut decision, player_one_salt);

        let player_one_decision_hash = hash::sha3_256(decision);
        let player_one_salt_hash = hash::sha3_256(player_one_salt);

        submit_decision(&player_one, 0, player_one_decision_hash, player_one_salt_hash, ctx);


        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash, ctx);

        reveal_decision(&player_one, 0, string::utf8(player_one_salt), ctx);
        timestamp::update_global_time_for_test_secs(3612, ctx);
        release_funds_after_expiration(account, 0, ctx);
        {
            let resouce_address = context::borrow_resource<ResouceAccountAddress>(
                ctx,
                @rooch_examples
            ).addr;

            {
                let state = context::borrow_mut_resource<State>(ctx, resouce_address);
                assert!(state.next_game_id == 1, 0);
            };
            assert!(account_coin_store::balance<WGBCOIN>(ctx, resouce_address) == 0, 40);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_one_address) == prize_pool_amount, 14);
            assert!(account_coin_store::balance<WGBCOIN>(ctx, player_two_address) == 0, 15);
        };
        context::drop_test_context(storage_context);
    }

    #[test]
    fun test_make_decision() {
        let decision_bytes = bcs::to_bytes(&DECISION_SPLIT);
        let salt = b"saltyyyyyy";
        vector::append(&mut decision_bytes, salt);

        let player_data = PlayerData {
            player_address: @0x123123123,
            salt_hash: option::some(hash::sha3_256(salt)),
            decision_hash: option::some(hash::sha3_256(decision_bytes)),
            decision: DECISION_NOT_MADE
        };

        let decision = make_decision(&mut player_data, &string::utf8(salt));
        assert!(decision == DECISION_SPLIT, 0);
        assert!(player_data.player_address == @0x123123123, 1);
        assert!(option::contains(&player_data.salt_hash, &hash::sha3_256(salt)), 2);
        assert!(option::contains(&player_data.decision_hash, &hash::sha3_256(decision_bytes)), 3);
        assert!(player_data.decision == DECISION_SPLIT, 4);
    }

    #[test]
    #[expected_failure(abort_code = 0x40001, location = std::option)]
    fun test_make_decision_salt_hash_is_none() {
        let decision_bytes = bcs::to_bytes(&DECISION_SPLIT);
        let salt = b"saltyyyyyy";
        vector::append(&mut decision_bytes, salt);

        let player_data = PlayerData {
            player_address: @0x123123123,
            salt_hash: option::none(),
            decision_hash: option::some(hash::sha3_256(decision_bytes)),
            decision: DECISION_NOT_MADE
        };

        make_decision(&mut player_data, &string::utf8(salt));
    }

    #[test]
    #[expected_failure(abort_code = 5, location = Self)]
    fun test_make_decision_incorrect_hash_value() {
        let decision_bytes = bcs::to_bytes(&DECISION_SPLIT);
        let salt = b"saltyyyyyy";
        vector::append(&mut decision_bytes, salt);

        let player_data = PlayerData {
            player_address: @0x123123123,
            salt_hash: option::some(hash::sha3_256(b"salt")),
            decision_hash: option::some(hash::sha3_256(decision_bytes)),
            decision: DECISION_NOT_MADE
        };

        make_decision(&mut player_data, &string::utf8(salt));
    }

    #[test]
    #[expected_failure(abort_code = 0x40001, location = std::option)]
    fun test_make_decision_decision_hash_is_none() {
        let decision_bytes = bcs::to_bytes(&DECISION_SPLIT);
        let salt = b"saltyyyyyy";
        vector::append(&mut decision_bytes, salt);

        let player_data = PlayerData {
            player_address: @0x123123123,
            salt_hash: option::some(hash::sha3_256(salt)),
            decision_hash: option::none(),
            decision: DECISION_NOT_MADE
        };

        make_decision(&mut player_data, &string::utf8(salt));
    }

    #[test]
    fun test_get_next_game_id() {
        let next_game_id_counter = 7328723;
        let next_game_id = get_next_game_id(&mut next_game_id_counter);

        assert!(next_game_id_counter == 7328724, 0);
        assert!(next_game_id == 7328723, 1);
    }
}
