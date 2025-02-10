module onchain_ai_chat::ai_service {
    use std::string::{Self, String};
    use std::vector;
    use std::option;
    use std::signer;
    use moveos_std::object::ObjectID;
    use moveos_std::account;
    use verity::oracles;
    use verity::registry;
    use rooch_framework::account_coin_store;
    use rooch_framework::gas_coin::RGas;

    use onchain_ai_chat::message::Message;
    use onchain_ai_chat::ai_request;

    friend onchain_ai_chat::ai_callback;
    friend onchain_ai_chat::room;

    const ORACLE_ADDRESS: address = @0x694cbe655b126e9e6a997e86aaab39e538abf30a8c78669ce23a98740b47b65d;
    const NOTIFY_CALLBACK: vector<u8> = b"ai_callback::process_response";
    /// Default gas allocation for notification callbacks 0.6 RGas
    const DEFAULT_NOTIFICATION_GAS: u256 = 60000000;
    const DEFAULT_ORACLE_FEE: u256 = 200000000;

    const AI_ORACLE_HEADERS: vector<u8> = b"{}";
    const AI_ORACLE_METHOD: vector<u8> = b"POST";

    /// The path to the message content in the oracle response
    /// We directly get the root, if we want to get the first choice we can use ".choices[].message.content"
    const AI_PICK: vector<u8> = b".";
    const AI_ORACLE_URL: vector<u8> = b"https://api.openai.com/v1/chat/completions";
    const MAX_HISTORY_MESSAGES: u64 = 10;

    const ErrorInvalidDepositAmount: u64 = 1;
    const ErrorInsufficientBalance: u64 = 2;

    struct PendingRequest has store, copy, drop {
        room_id: ObjectID,
        request_id: ObjectID,
    }

    struct Requests has key {
        pending: vector<PendingRequest>,
    }

    fun init() {
        let signer = moveos_std::signer::module_signer<Requests>();
        account::move_resource_to(&signer, Requests { 
            pending: vector::empty<PendingRequest>() 
        });
    }

    public(friend) fun request_ai_response(
        from: &signer,
        room_id: ObjectID,
        content: String,
        previous_messages: vector<Message>
    ) {
        let url = string::utf8(AI_ORACLE_URL);
        let method = string::utf8(AI_ORACLE_METHOD);
        let headers = string::utf8(AI_ORACLE_HEADERS);
        
        // Use ai_request to build the chat context
        let request = ai_request::new_chat_request(content, &previous_messages);
        let body = string::utf8(ai_request::to_json(&request));
        
        let pick = string::utf8(AI_PICK);
        let http_request = oracles::build_request(url, method, headers, body);
        
        let option_min_amount = registry::estimated_cost(ORACLE_ADDRESS, url, string::length(&body), 1024);
        
        let oracle_fee: u256 = if(option::is_some(&option_min_amount)) {
            option::destroy_some(option_min_amount)*40
        } else {
            DEFAULT_ORACLE_FEE
        };
        let from_addr = signer::address_of(from);
        let oracle_balance = oracles::get_user_balance(from_addr);
        if(oracle_balance < oracle_fee) {
            let gas_balance = account_coin_store::balance<RGas>(from_addr);
            assert!(gas_balance >= oracle_fee, ErrorInsufficientBalance);
            oracles::deposit_to_escrow(from, oracle_fee);
        };
        
        let request_id = oracles::new_request(
            http_request, 
            pick, 
            ORACLE_ADDRESS, 
            oracles::with_notify(@onchain_ai_chat, string::utf8(NOTIFY_CALLBACK))
        );

        // Store request information
        let requests = account::borrow_mut_resource<Requests>(@onchain_ai_chat);
        vector::push_back(&mut requests.pending, PendingRequest {
            room_id,
            request_id,
        });
        oracles::update_notification_gas_allocation(from, @onchain_ai_chat, string::utf8(NOTIFY_CALLBACK), DEFAULT_NOTIFICATION_GAS);
    }

    public(friend) fun get_pending_requests(): vector<PendingRequest> {
        let requests = account::borrow_resource<Requests>(@onchain_ai_chat);
        *&requests.pending
    }

    public(friend) fun remove_request(request_id: ObjectID) {
        let requests = account::borrow_mut_resource<Requests>(@onchain_ai_chat);
        let i = 0;
        let len = vector::length(&requests.pending);
        while (i < len) {
            let request = vector::borrow(&requests.pending, i);
            if (request.request_id == request_id) {
                vector::remove(&mut requests.pending, i);
                break
            };
            i = i + 1;
        };
    }

    public fun unpack_pending_request(request: PendingRequest): (ObjectID, ObjectID) {
        (request.room_id, request.request_id)
    }

    public fun get_user_oracle_fee_balance(user_addr: address): u256 {
        oracles::get_user_balance(user_addr)
    }

    public entry fun withdraw_user_oracle_fee(caller: &signer, amount: u256) {
        oracles::withdraw_from_escrow(caller, amount)
    }

    public entry fun withdraw_all_user_oracle_fee(caller: &signer) {
        let balance = oracles::get_user_balance(signer::address_of(caller));
        oracles::withdraw_from_escrow(caller, balance)
    }

    public entry fun deposit_user_oracle_fee(caller: &signer, amount: u256) {
        // Check user's RGas balance
        let caller_addr = signer::address_of(caller);
        let gas_balance = account_coin_store::balance<RGas>(caller_addr);
        assert!(gas_balance >= amount, ErrorInsufficientBalance);
        
        oracles::deposit_to_escrow(caller, amount)
    }

    #[test_only]
    use onchain_ai_chat::message;

    #[test]
    fun test_request_ai_response() {
        use std::string;
        
        // Test basic request creation
        let messages = vector::empty<Message>();
        vector::push_back(&mut messages, message::new_message(0, @0x1, string::utf8(b"Hi"), message::type_user()));
        let content = string::utf8(b"Hello AI");
        
        // Create request and verify JSON structure
        let request = ai_request::new_chat_request(content, &messages);
        let body = string::utf8(ai_request::to_json(&request));
        
        // Verify JSON structure contains required fields
        assert!(string::index_of(&body, &string::utf8(b"gpt-4o")) != 18446744073709551615, 1);
        assert!(string::index_of(&body, &string::utf8(b"messages")) != 18446744073709551615, 2);
        assert!(string::index_of(&body, &string::utf8(b"user")) != 18446744073709551615, 3);
    }

    #[test]
    fun test_oracle_fee_operations() {
        oracles::init_for_test();

        // Initialize test accounts
        let alice = account::create_signer_for_testing(@0x77);
        let alice_addr = signer::address_of(&alice);

        // Setup test account with initial RGas
        let fee_amount: u256 = 1000000000; // 10 RGas
        rooch_framework::gas_coin::faucet_entry(&alice, fee_amount);

        // Test Case 1: Check initial balance
        {
            let initial_balance = get_user_oracle_fee_balance(alice_addr);
            assert!(initial_balance == 0, 1);
        };

        // Test Case 2: Deposit and check balance
        {
            deposit_user_oracle_fee(&alice, fee_amount);
            let balance = get_user_oracle_fee_balance(alice_addr);
            assert!(balance == fee_amount, 2);
        };

        // Test Case 3: Partial withdrawal
        {
            let withdraw_amount = fee_amount / 2;
            withdraw_user_oracle_fee(&alice, withdraw_amount);
            let balance = get_user_oracle_fee_balance(alice_addr);
            assert!(balance == withdraw_amount, 3);
        };

        // Test Case 4: Withdraw all remaining balance
        {
            withdraw_all_user_oracle_fee(&alice);
            let balance = get_user_oracle_fee_balance(alice_addr);
            assert!(balance == 0, 4);
        };
    }
}