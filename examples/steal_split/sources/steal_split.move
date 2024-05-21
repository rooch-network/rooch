// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

module rooch_examples::rooch_examples {
    use std::bcs;
    use std::hash;
    use std::option::{Self, Option};
    use std::signer;
    use std::string::{Self, String};
    use std::vector;
    use moveos_std::account;
    use moveos_std::account::Account;

    use moveos_std::event::Self;
    use moveos_std::simple_map::{Self, SimpleMap};
    
    use moveos_std::object;
    use rooch_framework::coin;
    use rooch_framework::account_coin_store;
    use rooch_examples::timestamp;
    use rooch_framework::account as account_entry;

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

    // TODO: remove resource account address
    // struct ResouceAccountAddress has key {
    //     addr: address
    // }

    // TODO: Account holder holds an account of the resource
    struct AccountHolder {
        account: Object<Account>,
    }

    struct State has key {
        next_game_id: u128,
        games: SimpleMap<u128, Game>,
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

    struct CreateGameEvent has store, drop, copy {
        game_id: u128,
        prize_pool_amount: u256,
        player_one_address: address,
        player_two_address: address,
        expiration_timestamp_in_seconds: u64,
        event_creation_timestamp_in_seconds: u64
    }


    struct SubmitDecisionEvent has store, drop, copy {
        game_id: u128,
        player_address: address,
        decision_hash: vector<u8>,
        salt_hash: vector<u8>,
        event_creation_timestamp_in_seconds: u64
    }

    struct RevealDecisionEvent has store, drop, copy {
        game_id: u128,
        player_address: address,
        decision: u64,
        event_creation_timestamp_in_seconds: u64
    }

    struct ConcludeGameEvent has store, drop, copy {
        game_id: u128,
        player_one_decision: u64,
        player_two_decision: u64,
        prize_pool_amount: u256,
        event_creation_timestamp_in_seconds: u64
    }

    struct ReleaseFundsAfterExpirationEvent has store, drop, copy {
        game_id: u128,
        player_one_decision: u64,
        player_two_decision: u64,
        prize_pool_amount: u256,
        event_creation_timestamp_in_seconds: u64
    }

    /// ----------TODO---------- ///
    /// Scheme identifier used when hashing an account's address together with a seed to derive the address (not the
    /// authentication key) of a resource account. This is an abuse of the notion of a scheme identifier which, for now,
    /// serves to domain separate hashes used to derive resource account addresses from hashes used to derive
    /// authentication keys. Without such separation, an adversary could create (and get a signer for) a resource account
    /// whose address matches an existing address of a MultiEd25519 wallet.
    // const SCHEME_DERIVE_RESOURCE_ACCOUNT: u8 = 255;

    // /// A resource account is used to manage resources independent of an account managed by a user.
    // /// In Rooch a resource account is created based upon the sha3 256 of the source's address and additional seed data.
    // /// A resource account can only be created once
    // public fun create_resource_account(source: &signer): (signer, SignerCapability) {
    //     let source_addr = signer::address_of(source);
    //     let seed = account::generate_seed_bytes(&source_addr);
    //     let resource_addr = create_resource_address(&source_addr, seed);
    //     assert!(!is_resource_account(resource_addr), ErrorAccountIsAlreadyResourceAccount);
    //     let resource_signer = if (exists_at(resource_addr)) {
    //         let object_id = account_object_id(resource_addr);
    //         let obj = object::borrow_object<Account>(object_id);
    //         let account = object::borrow<Account>(obj);
    //         assert!(account.sequence_number == 0, ErrorResourceAccountAlreadyUsed);
    //         create_signer(resource_addr)
    //     } else {
    //         create_account_unchecked(resource_addr)
    //     };

    //     move_resource_to<ResourceAccount>(&resource_signer,ResourceAccount {});

    //     let signer_cap = SignerCapability { addr: resource_addr };

    //     account_authentication::init_authentication_keys(&resource_signer);
    //     account_coin_store::init_account_coin_stores(&resource_signer);
    //     (resource_signer, signer_cap)
    // }

    // public fun is_resource_account(addr: address): bool {
    //     exists_resource<ResourceAccount>(addr)
    // }

    // /// This is a helper function to compute resource addresses. Computation of the address
    // /// involves the use of a cryptographic hash operation and should be use thoughtfully.
    // fun create_resource_address(source: &address, seed: vector<u8>): address {
    //     let bytes = bcs::to_bytes(source);
    //     vector::append(&mut bytes, seed);
    //     vector::push_back(&mut bytes, SCHEME_DERIVE_RESOURCE_ACCOUNT);
    //     bcs::to_address(hash::sha3_256(bytes))
    // }

    // #[test]
    // fun test_create_resource_account()  {
    //     let alice_addr = @123456;
    //     let alice = create_account_for_testing(alice_addr);
    //     let (resource_account, resource_account_cap) = create_resource_account(&alice);
    //     let signer_cap_addr = get_signer_capability_address(&resource_account_cap);
    //     move_resource_to<CapResponsbility>(
    //         &resource_account,
    //         CapResponsbility {
    //             cap: resource_account_cap
    //         }
    //     );

    //     let resource_addr = signer::address_of(&resource_account);
    //     std::debug::print(&100100);
    //     std::debug::print(&resource_addr);
    //     assert!(resource_addr != signer::address_of(&alice), 106);
    //     assert!(resource_addr == signer_cap_addr, 107);
    // }

    // //TODO figure out why this test should failed
    // #[test(sender=@0x42, resource_account=@0xbb6e573f7feb9d8474ac20813fc086cc3100b8b7d49c246b0f4aee8ea19eaef4)]
    // #[expected_failure(abort_code = ErrorResourceAccountAlreadyUsed, location = Self)]
    // fun test_failure_create_resource_account_wrong_sequence_number(sender: address, resource_account: address){
    //     {
    //         create_account_for_testing(resource_account);
    //         increment_sequence_number_internal(resource_account);
    //     };
    //     let sender_signer = create_account_for_testing(sender);
    //     let (signer, cap) = create_resource_account(&sender_signer);
    //     move_resource_to<CapResponsbility>(
    //         &signer,
    //         CapResponsbility {
    //             cap
    //         }
    //     );
    // }
    /// ----------TODO---------- ///

    fun init(account: &signer) {
        let account_address = signer::address_of(account);
        account::create_account_object(&account_address);
        let account_obj = object::borrow_mut_object<AccountHolder>(&account, account::account_object_id(account_address));
        // account_authentication::init_authentication_keys(&resource_signer);
        // account_coin_store::init_account_coin_stores(&resource_signer);

        let coin_info = coin::register_extend<WGBCOIN>(string::utf8(b"WGBCOIN"),string::utf8(b"WGB"), 8);

        let coin = coin::mint_extend<WGBCOIN>(&mut coin_info, 1000 * 1000 * 1000);
        account_coin_store::do_accept_coin<WGBCOIN>(account);
        account_coin_store::do_accept_coin<WGBCOIN>(&signer);
        account_coin_store::deposit_extend(account_address, coin);
        object::transfer(coin_info, account_address);
        account::account_move_resource_to(&account_obj, State {
            next_game_id: 0,
            games: simple_map::create()
        });
    }

    public entry fun create_game(
        account: &signer,
        prize_pool_amount: u256,
        player_one_address: address,
        player_two_address: address,
    ) {
        let account_address = check_if_state_exists(account);
        let now = timestamp::now_seconds();
        check_if_signer_is_contract_deployer(account);
        // let resouce_address = account::borrow_resource<ResouceAccountAddress>(@rooch_examples).addr;
        let next_game_id = {
            let state_mut_ref = account::account_borrow_mut_resource<State>(account_address);
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
            let state_mut_ref = account::account_borrow_mut_resource<State>(account_address);
            simple_map::add(&mut state_mut_ref.games, next_game_id, new_game);
        };
        account_coin_store::transfer<WGBCOIN>(account, account_address, prize_pool_amount);
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
    ) {
        let player_address = check_if_state_exists(player);
        let now = timestamp::now_seconds();
        // let resouce_address = account::borrow_resource<ResouceAccountAddress>(@rooch_examples).addr;
        let state_mut_ref = account::account_borrow_mut_resource<State>(player_address);
        check_if_game_exists(&state_mut_ref.games, &game_id);
        let game_mut_ref = simple_map::borrow_mut(&mut state_mut_ref.games, &game_id);
        check_if_player_participates_in_the_game(player, game_mut_ref);
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
    ) {
        let player_address = check_if_state_exists(player);
        let now = timestamp::now_seconds();
        // let resouce_address = account::borrow_resource<ResouceAccountAddress>(@rooch_examples).addr;
        let (game_id, decision) = {
            let state_mut_ref = account::account_borrow_mut_resource<State>(player_address);

            check_if_game_exists(&state_mut_ref.games, &game_id);
            let game_mut_ref = simple_map::borrow_mut(&mut state_mut_ref.games, &game_id);
            check_if_player_participates_in_the_game(player, game_mut_ref);
            check_if_both_players_have_a_decision_submitted(game_mut_ref);
            let (current_player_data_mut_ref, another_player_data_mut_ref) = if (game_mut_ref.player_one.player_address == player_address) {
                (&mut game_mut_ref.player_one, &mut game_mut_ref.player_two)
            }
            else {
                (&mut game_mut_ref.player_two, &mut game_mut_ref.player_one)
            };
            let decision = make_decision(current_player_data_mut_ref, &salt);

            if (another_player_data_mut_ref.decision != DECISION_NOT_MADE) {
                let (_, game) = simple_map::remove(&mut state_mut_ref.games, &game_id);
                if ((game.player_one.decision == game.player_two.decision) && (game.player_one.decision == DECISION_SPLIT)) {
                    let player_one_amount = game.prize_pool_amount / 2;
                    let player_two_amount = game.prize_pool_amount - player_one_amount;
                    account_coin_store::transfer<WGBCOIN>(
                        game.player_one.player_address,
                        player_one_amount
                    );
                    account_coin_store::transfer<WGBCOIN>(
                        game.player_two.player_address,
                        player_two_amount
                    );
                }else if ((game.player_one.decision != game.player_two.decision)) {
                    let steal_player_address = if (game.player_one.decision == DECISION_SPLIT) {
                        game.player_two.player_address
                    }else {
                        game.player_one.player_address
                    };
                    account_coin_store::transfer<WGBCOIN>(steal_player_address, game.prize_pool_amount);
                }else {
                    account_coin_store::transfer<WGBCOIN>(@rooch_examples, game.prize_pool_amount);
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
            (game_id, decision)
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

    public entry fun release_funds_after_expiration(account: &signer, game_id: u128) {
        let account_address = check_if_state_exists(account);
        let now = timestamp::now_seconds();
        // let resouce_address = account::borrow_resource<ResouceAccountAddress>(@rooch_examples).addr;
        let game = {
            let state_mut_ref = account::account_borrow_mut_resource<State>(account_address);
            check_if_game_exists(&state_mut_ref.games, &game_id);

            let (_, game) = simple_map::remove(&mut state_mut_ref.games, &game_id);
            game
        };

        check_if_game_expired(&game);

        if (game.player_one.decision == game.player_two.decision) {
            account_coin_store::transfer<WGBCOIN>(
                @rooch_examples, 
                game.prize_pool_amount
            );
        }else if (game.player_one.decision != DECISION_NOT_MADE) {
            account_coin_store::transfer<WGBCOIN>(
                game.player_one.player_address,
                game.prize_pool_amount
            );
        }else {
            account_coin_store::transfer<WGBCOIN>(
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

    fun check_if_state_exists(signer: &signer): address {
        let signer_address = signer::address_of(signer);
        assert!(account::account_exists_resource<AccountHolder>(signer_addr), ErrorStateIsNotInitialized);
        assert!(account::exists_resource<State>(signer_addr), ErrorStateIsNotInitialized);
        signer_address
    }

    fun check_if_signer_is_contract_deployer(signer: &signer) {
        assert!(signer::address_of(signer) == @rooch_examples, ErrorSignerIsNotDeployer);
    }

    fun check_if_account_has_enough_apt_coins(account: &signer, amount: u256) {
        assert!(account_coin_store::balance<WGBCOIN>(signer::address_of(account)) >= amount, ErrorSignerHasInsufficientAptBalance);
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

    fun check_if_game_expired(game: &Game) {
        assert!(game.expiration_timestamp_in_seconds <= timestamp::now_seconds(), ErrorGameNotExpiredYet);
    }


    #[test]
    fun test_init() {
        genesis::init_for_test();
        

        let account = &account_entry::create_account_for_testing(@rooch_examples);
        timestamp::set_time_has_started_for_testing();

        init(account);
        let account_address = signer::address_of(account);

        let state = account::account_borrow_mut_resource<State>(account_address);
        assert!(state.next_game_id == 0, 0);
        assert!(simple_map::length(&state.games) == 0, 1);
        assert!(account_coin_store::is_accept_coin<WGBCOIN>(account_address), 12);
        
    }

    #[test]
    #[expected_failure(abort_code = 6, location = moveos_std::account)]
    fun test_init_again() {
        genesis::init_for_test();
        
        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();

        init(account);
        init(account);
        
    }

    #[test]
    fun test_create_game() {
        genesis::init_for_test();
        
        //let account = &account_entry::create_account_for_testing(moveos_std::tx_context::sender());
        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();
 
        init(account);
        let account_address = signer::address_of(account);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;

        
        timestamp::update_global_time_for_test_secs(10);
        create_game(account, prize_pool_amount, player_one_address, player_two_address);

        let state = account::account_borrow_mut_resource<State>(account_address);
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

        assert!(account_coin_store::balance<WGBCOIN>(account_address) == prize_pool_amount, 23);

        
    }


    #[test]
    fun test_submit_decision() {
        genesis::init_for_test();
        

        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();

        init(account);
        let account_address = signer::address_of(account);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10);

        let (player_one, _player_two) = {
            (account_entry::create_account_for_testing(player_one_address), account_entry::create_account_for_testing(
                
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address);

        let salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut decision, salt);

        let decision_hash = hash::sha3_256(decision);
        let salt_hash = hash::sha3_256(salt);

        submit_decision(&player_one, 0, decision_hash, salt_hash);

        let state = account::account_borrow_mut_resource<State>(account_address);

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

        assert!(account_coin_store::balance<WGBCOIN>(account_address) == prize_pool_amount, 24);
        
    }


    #[test]
    #[expected_failure(abort_code = ErrorPlayerHasDecisionSubmitted, location = Self)]
    fun test_submit_decision_player_one_has_a_decision_submitted() {
        genesis::init_for_test();
        

        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();

        init(account);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10);

        let (player_one, _player_two) = {
            (account_entry::create_account_for_testing(player_one_address), account_entry::create_account_for_testing(
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address);
        let salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut decision, salt);

        let decision_hash = hash::sha3_256(decision);
        let salt_hash = hash::sha3_256(salt);

        submit_decision(&player_one, 0, decision_hash, salt_hash);
        submit_decision(&player_one, 0, decision_hash, salt_hash);
        
    }


    #[test]
    fun test_reveal_decision_split() {
        genesis::init_for_test();
        

        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();

        init(account);
        let account_address = signer::address_of(account);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10);

        let (player_one, player_two) = {
            (account_entry::create_account_for_testing(player_one_address), account_entry::create_account_for_testing(
                
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address);

        account_coin_store::do_accept_coin<WGBCOIN>(&player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(&player_two);
        let player_one_salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut decision, player_one_salt);

        let player_one_decision_hash = hash::sha3_256(decision);
        let player_one_salt_hash = hash::sha3_256(player_one_salt);

        submit_decision(&player_one, 0, player_one_decision_hash, player_one_salt_hash);


        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash);

        reveal_decision(&player_one, 0, string::utf8(player_one_salt));
        {
            {
                let state = account::account_borrow_mut_resource<State>(account_address);
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

            assert!(account_coin_store::balance<WGBCOIN>(account_address) == prize_pool_amount, 23);
            assert!(account_coin_store::balance<WGBCOIN>(player_one_address) == 0, 24);
            assert!(account_coin_store::balance<WGBCOIN>(player_two_address) == 0, 25);

            reveal_decision(&player_two, 0, string::utf8(player_two_salt));
            {
                let state = account::account_borrow_mut_resource<State>(account_address);
                assert!(state.next_game_id == 1, 28);
                assert!(simple_map::length(&state.games) == 0, 29);
            };

            assert!(account_coin_store::balance<WGBCOIN>(account_address) == 0, 40);
            assert!(account_coin_store::balance<WGBCOIN>(player_one_address) == prize_pool_amount / 2, 42);
            assert!(account_coin_store::balance<WGBCOIN>(player_two_address) == prize_pool_amount / 2, 43);
            
        }
    }

    #[test]
    fun test_reveal_decision_player_one_steals() {
        genesis::init_for_test();
        

        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();

        init(account);
        let account_address = signer::address_of(account);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10);

        let (player_one, player_two) = {
            (account_entry::create_account_for_testing(player_one_address), account_entry::create_account_for_testing(
                
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address);

        account_coin_store::do_accept_coin<WGBCOIN>(&player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(&player_two);
        let player_one_salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut decision, player_one_salt);

        let player_one_decision_hash = hash::sha3_256(decision);
        let player_one_salt_hash = hash::sha3_256(player_one_salt);

        submit_decision(&player_one, 0, player_one_decision_hash, player_one_salt_hash);


        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash);

        reveal_decision(&player_one, 0, string::utf8(player_one_salt));
        reveal_decision(&player_two, 0, string::utf8(player_two_salt));
        {
            {
                let state = account::account_borrow_mut_resource<State>(account_address);
                assert!(state.next_game_id == 1, 0);
            };
            assert!(account_coin_store::balance<WGBCOIN>(account_address) == 0, 40);
            assert!(account_coin_store::balance<WGBCOIN>(player_one_address) == prize_pool_amount, 42);
            assert!(account_coin_store::balance<WGBCOIN>(player_two_address) == 0, 43);
        };
        
    }

    #[test]
    fun test_reveal_decision_player_two_steals() {
        genesis::init_for_test();
        

        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();

        init(account);
        let account_address = signer::address_of(account);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10);

        let (player_one, player_two) = {
            (account_entry::create_account_for_testing(player_one_address), account_entry::create_account_for_testing(
                
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address);

        account_coin_store::do_accept_coin<WGBCOIN>(&player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(&player_two);
        let player_one_salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_SPLIT);
        vector::append(&mut decision, player_one_salt);

        let player_one_decision_hash = hash::sha3_256(decision);
        let player_one_salt_hash = hash::sha3_256(player_one_salt);

        submit_decision(&player_one, 0, player_one_decision_hash, player_one_salt_hash);


        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash);

        reveal_decision(&player_one, 0, string::utf8(player_one_salt));
        reveal_decision(&player_two, 0, string::utf8(player_two_salt));

        {

            {
                let state = account::account_borrow_mut_resource<State>(account_address);
                assert!(state.next_game_id == 1, 0);
            };
            assert!(account_coin_store::balance<WGBCOIN>(account_address) == 0, 40);
            assert!(account_coin_store::balance<WGBCOIN>(player_one_address) == 0, 42);
            assert!(account_coin_store::balance<WGBCOIN>(player_two_address) == prize_pool_amount, 43);
        };
        
    }

    #[test]
    fun test_reveal_decision_both_players_steal() {
        genesis::init_for_test();
        

        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();

        init(account);
        let account_address = signer::address_of(account);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10);

        let (player_one, player_two) = {
            (account_entry::create_account_for_testing(player_one_address), account_entry::create_account_for_testing(
                
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address);

        account_coin_store::do_accept_coin<WGBCOIN>(&player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(&player_two);
        let player_one_salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut decision, player_one_salt);

        let player_one_decision_hash = hash::sha3_256(decision);
        let player_one_salt_hash = hash::sha3_256(player_one_salt);

        submit_decision(&player_one, 0, player_one_decision_hash, player_one_salt_hash);


        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash);

        reveal_decision(&player_one, 0, string::utf8(player_one_salt));
        reveal_decision(&player_two, 0, string::utf8(player_two_salt));

        {
            {
                let state = account::account_borrow_mut_resource<State>(account_address);
                assert!(state.next_game_id == 1, 0);
            };
            assert!(account_coin_store::balance<WGBCOIN>(account_address) == 0, 40);
            assert!(account_coin_store::balance<WGBCOIN>(player_one_address) == 0, 42);
            assert!(account_coin_store::balance<WGBCOIN>(player_two_address) == 0, 43);
        };
        
    }


    #[test]
    #[expected_failure(abort_code = ErrorBothPlayersDoNotHaveDecisionsSubmitted, location = Self)]
    fun test_reveal_decision_player_one_does_not_have_a_decision_submitted() {
        genesis::init_for_test();
        

        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();

        init(account);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10);

        let (player_one, player_two) = {
            (account_entry::create_account_for_testing(player_one_address), account_entry::create_account_for_testing(
                
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address);

        account_coin_store::do_accept_coin<WGBCOIN>(&player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(&player_two);

        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash);
        reveal_decision(&player_two, 0, string::utf8(player_two_salt));
        
    }

    #[test]
    fun test_release_funds_after_expiration_transfer_to_creator() {
        genesis::init_for_test();
        

        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();

        init(account);
        let account_address = signer::address_of(account);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10);

        let (player_one, player_two) = {
            (account_entry::create_account_for_testing(player_one_address), account_entry::create_account_for_testing(
                
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address);

        account_coin_store::do_accept_coin<WGBCOIN>(&player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(&player_two);

        timestamp::update_global_time_for_test_secs(3612);
        release_funds_after_expiration(account, 0);
        {
            {
                let state = account::account_borrow_mut_resource<State>(account_address);
                assert!(state.next_game_id == 1, 0);
            };
            assert!(account_coin_store::balance<WGBCOIN>(account_address) == 0, 13);
            assert!(account_coin_store::balance<WGBCOIN>(player_one_address) == 0, 14);
            assert!(account_coin_store::balance<WGBCOIN>(player_two_address) == 0, 15);
        };
        
    }

    #[test]
    fun test_release_funds_after_expiration_transfer_to_player_one() {
        genesis::init_for_test();
        

        let account = &account_entry::create_account_for_testing(@rooch_examples);
        let _move_os = &account_entry::create_account_for_testing(@moveos_std);
        timestamp::set_time_has_started_for_testing();

        init(account);
        let account_address = signer::address_of(account);

        let prize_pool_amount:u256 = 1000;
        let player_one_address = @0xACE;
        let player_two_address = @0xCAFE;
        timestamp::update_global_time_for_test_secs(10);

        let (player_one, player_two) = {
            (account_entry::create_account_for_testing(player_one_address), account_entry::create_account_for_testing(
                
                player_two_address
            ))
        };

        create_game(account, prize_pool_amount, player_one_address, player_two_address);

        account_coin_store::do_accept_coin<WGBCOIN>(&player_one);
        account_coin_store::do_accept_coin<WGBCOIN>(&player_two);
        let player_one_salt = b"saltsaltsalt";
        let decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut decision, player_one_salt);

        let player_one_decision_hash = hash::sha3_256(decision);
        let player_one_salt_hash = hash::sha3_256(player_one_salt);

        submit_decision(&player_one, 0, player_one_decision_hash, player_one_salt_hash);


        let player_two_salt = b"saltyyyy";
        let player_two_decision = bcs::to_bytes(&DECISION_STEAL);
        vector::append(&mut player_two_decision, player_two_salt);

        let player_two_decision_hash = hash::sha3_256(player_two_decision);
        let player_two_salt_hash = hash::sha3_256(player_two_salt);

        submit_decision(&player_two, 0, player_two_decision_hash, player_two_salt_hash);

        reveal_decision(&player_one, 0, string::utf8(player_one_salt));
        timestamp::update_global_time_for_test_secs(3612);
        release_funds_after_expiration(account, 0);
        {
            {
                let state = account::account_borrow_mut_resource<State>(account_address);
                assert!(state.next_game_id == 1, 0);
            };
            assert!(account_coin_store::balance<WGBCOIN>(account_address) == 0, 40);
            assert!(account_coin_store::balance<WGBCOIN>(player_one_address) == prize_pool_amount, 14);
            assert!(account_coin_store::balance<WGBCOIN>(player_two_address) == 0, 15);
        };
        
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
    #[expected_failure(abort_code = 262145, location = std::option)]
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
    #[expected_failure(abort_code = ErrorIncorrectHashValue, location = Self)]
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
    #[expected_failure(abort_code = 262145, location = std::option)]
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
