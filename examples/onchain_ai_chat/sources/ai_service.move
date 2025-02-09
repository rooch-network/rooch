module onchain_ai_chat::ai_service {
    use std::string::{Self, String};
    use std::vector;
    use moveos_std::object::ObjectID;
    use moveos_std::account;
    use verity::oracles;
    use rooch_framework::gas_coin::RGas;
    use rooch_framework::account_coin_store;

    friend onchain_ai_chat::ai_callback;
    friend onchain_ai_chat::room;

    const ORACLE_ADDRESS: address = @0x694cbe655b126e9e6a997e86aaab39e538abf30a8c78669ce23a98740b47b65d;
    const NOTIFY_CALLBACK: vector<u8> = b"ai_callback::process_response";
    /// Default gas allocation for notification callbacks 0.6 RGas
    const DEFAULT_NOTIFICATION_GAS: u256 = 60000000;

    const AI_ORACLE_HEADERS: vector<u8> = b"{}";
    const AI_ORACLE_METHOD: vector<u8> = b"POST";

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
    ) {
        //TODO eliminate the gas fee
        let oracle_fee: u256 = 1000000000;
        let url = string::utf8(b"https://api.openai.com/v1/chat/completions");
        let method = string::utf8(AI_ORACLE_METHOD);
        let headers = string::utf8(AI_ORACLE_HEADERS);
        
        let body = string::utf8(b"{\"model\": \"gpt-4o\", \"messages\": [{\"role\": \"user\", \"content\": \"");
        string::append(&mut body, content);
        string::append(&mut body, string::utf8(b"\"}], \"temperature\": 0.7}"));
        
        //let pick = string::utf8(b".choices[].message.content");
        let pick = string::utf8(b".");
        let http_request = oracles::build_request(url, method, headers, body);
        let payment = account_coin_store::withdraw<RGas>(from, oracle_fee);
        
        let request_id = oracles::new_request_with_payment(
            http_request, 
            pick, 
            ORACLE_ADDRESS, 
            oracles::with_notify(@onchain_ai_chat, string::utf8(NOTIFY_CALLBACK)),
            payment
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
}