// Copyright (c) Usher Labs
// SPDX-License-Identifier: LGPL-2.1

// ? This module is an example caller used to demonstrate how to deploy Contracts on Rooch that integrate with Verity Move Oracles.
// ? Please keep aware of the OPTIONAL section in this module.
module verity_oracle_example::example_caller {
    use moveos_std::event;
    use moveos_std::account;
    use moveos_std::object::{ObjectID};
    use std::option::{Self, Option};
    use std::vector;
    use std::string::String;
    use verity::oracles::{Self as Oracles};
    use rooch_framework::gas_coin::RGas;
    use rooch_framework::account_coin_store;
    #[test_only]
    use verity::oracles;

    struct GlobalParams has key {
       pending_requests: vector<ObjectID>,
    }

    #[test_only]
    public fun init_for_test(){
        oracles::init_for_test();
        init();
    }


    // ? ------ OPTIONAL ------
    // ? This is totally OPTIONAL
    struct RequestFulfilledEvent has copy, drop {
      request_url: String,
      request_method: String,
      response: Option<String>,
    }
    // \ ------ OPTIONAL ------

    // Initiate the module with an empty vector of pending requests
    // Requests are managed in the caller to prevent other modules from impersonating the calling module, and spoofing new data.
    fun init(){
      // let params = account::borrow_mut_resource<GlobalParams>(@verity_test_foreign_module); // account::borrow_mut_resource  in init throws an error on deployment
      // params.pending_requests = vector::empty<ObjectID>();
      let signer = moveos_std::signer::module_signer<GlobalParams>();
      account::move_resource_to(&signer, GlobalParams { pending_requests: vector::empty<ObjectID>() });
    }

    public entry fun request_data(
        from: &signer,
        url: String,
        method: String,
        headers: String,
        body: String,
        pick: String,
        oracle: address,
        amount: u256
    ) {
        let http_request = Oracles::build_request(url, method, headers, body);

        // We're passing the address and function identifier of the recipient address. in this from <module_name>::<function_name>
        // If you do not want to pay for the Oracle to notify your contract, you can pass in option::none() as the argument.
        let payment = account_coin_store::withdraw<RGas>(from, amount);
        let request_id = Oracles::new_request_with_payment(http_request, pick, oracle, Oracles::with_notify(@verity_test_foreign_module, b"example_caller::receive_data"),payment);
        // let no_notify_request_id = Oracles::new_request(http_request, pick, oracle, Oracles::without_notify());
        let params = account::borrow_mut_resource<GlobalParams>(@verity_test_foreign_module);
        vector::push_back(&mut params.pending_requests, request_id);
    }

    // This notify function is called by the Oracle.
    // ! It must not include parameters, or return arguments.
    public entry fun receive_data() {
        let params = account::borrow_mut_resource<GlobalParams>(@verity_test_foreign_module);
        let pending_requests = params.pending_requests;

        let i = 0;
        while (i < vector::length(&pending_requests)) {
            let request_id = vector::borrow(&pending_requests, i);
            // Remove the fulfilled request from the pending_requests vector
            // This ensures unfulfilled requests are retained in the vector
            if (option::is_some(&Oracles::get_response(request_id))) {
                vector::remove(&mut params.pending_requests, i);
                // Decrement i to account for the removed element
                if (i > 0) {
                    i = i - 1;
                };

                // ? ------ OPTIONAL ------
                let request_url = Oracles::get_request_params_url(request_id);
                let request_method = Oracles::get_request_params_method(request_id);
                let response = Oracles::get_response(request_id);
                // For each fulfilment, emit an event
                event::emit(RequestFulfilledEvent {
                  request_url,
                  request_method,
                  response,
                });
                // \ ------ OPTIONAL ------
            };

            i = i + 1;
        };
    }

    #[view]
    public fun pending_requests_count(): u64 {
        let params = account::borrow_resource<GlobalParams>(@verity_test_foreign_module);
        vector::length(&params.pending_requests)
    }
}

#[test_only]
module verity_oracle_example::test_foreign_module {
    use moveos_std::signer;
    use verity_test_foreign_module::example_caller::{Self, request_data, pending_requests_count};
    use rooch_framework::gas_coin;
    use verity::registry;
    use std::vector;



    #[test_only]
    struct Test has key {}

    #[test_only]
    struct TestOrchestrator has key {}

    #[test_only]
    fun setup_test() {
        example_caller::init_for_test();
    }

    #[test]
    fun test_request_data_basic() {
        setup_test();
        let test_signer = moveos_std::signer::module_signer<Test>();
        let test_orchestrator = moveos_std::signer::module_signer<TestOrchestrator>();


        
        // Setup test parameters
        let url = std::string::utf8(b"https://api.test.com");
        let method = std::string::utf8(b"GET");
        let headers = std::string::utf8(b"Content-Type: application/json");
        let body = std::string::utf8(b"");
        let pick = std::string::utf8(b"$.data");
        let amount = 100000u256;

        registry::add_supported_url(&test_orchestrator, url, 100, 0, 1, 0);
        

        // Fund the test account
        gas_coin::faucet_entry(&test_signer, amount);

        // Make request
        request_data(&test_signer, url, method, headers, body, pick, signer::address_of(&test_orchestrator), amount);

        // Verify request was stored
        
        assert!(pending_requests_count() == 1, 0);
    }


    #[test]
    fun test_multiple_requests() {
        setup_test();
        let test_signer = moveos_std::signer::module_signer<Test>();
        let test_orchestrator = moveos_std::signer::module_signer<TestOrchestrator>();


        // Setup test parameters for multiple requests
        let urls = vector[
            std::string::utf8(b"https://api.test.com/2/test/id2"),
            std::string::utf8(b"https://api.test.com/2/my_profile"),
            std::string::utf8(b"https://api.test.com/2/test")
        ];
        let method = std::string::utf8(b"GET");
        let headers = std::string::utf8(b"Content-Type: application/json");
        let body = std::string::utf8(b"");
        let pick = std::string::utf8(b"$.data");
        let amount = 100u256;

        registry::add_supported_url(&test_orchestrator, std::string::utf8(b"https://api.test.com/2/"), 100, 0, 1, 0);


        // Fund the test account with enough for multiple requests
        let total_amount = amount * 4;
        gas_coin::faucet_entry(&test_signer, total_amount);

        // Make multiple requests
        let i = 0;
        while (i < vector::length(&urls)) {
            let url = *vector::borrow(&urls, i);
            request_data(&test_signer, url, method, headers, body, pick, signer::address_of(&test_orchestrator), amount);
            i = i + 1;
        };

        // Verify all requests were stored
        assert!(pending_requests_count() == 3, 2);

    }

    #[test]
    #[expected_failure(abort_code = rooch_framework::coin_store::ErrorInsufficientBalance, location = rooch_framework::coin_store)] // Adjust abort code as needed
    fun test_insufficient_funds() {
        setup_test();
        let test_signer = moveos_std::signer::module_signer<Test>();
        
        let url = std::string::utf8(b"https://api.test.com");
        let method = std::string::utf8(b"GET");
        let headers = std::string::utf8(b"Content-Type: application/json");
        let body = std::string::utf8(b"");
        let pick = std::string::utf8(b"$.data");
        let oracle = @0x1;
        let amount = 100u256;



        gas_coin::faucet_entry(&test_signer, amount/10);
        request_data(&test_signer, url, method, headers, body, pick, oracle, amount);
    }

    #[test]
    fun test_request_with_body() {
        setup_test();
        let test_signer = moveos_std::signer::module_signer<Test>();
        let test_orchestrator = moveos_std::signer::module_signer<TestOrchestrator>();
        
        let url = std::string::utf8(b"https://api.test.com/yud");
        let method = std::string::utf8(b"POST");
        let headers = std::string::utf8(b"Content-Type: application/json");
        let body = std::string::utf8(b"{\"key\":\"value\"}");
        let pick = std::string::utf8(b".data");
        let oracle = signer::address_of(&test_orchestrator);
        let amount = 1000u256;


        gas_coin::faucet_entry(&test_signer, amount);


        registry::add_supported_url(&test_orchestrator, std::string::utf8(b"https://api.test.com/"), 100, 0, 1, 0);
        request_data(&test_signer, url, method, headers, body, pick, oracle, amount);

        assert!(pending_requests_count() == 1, 4);
        
    }
}
