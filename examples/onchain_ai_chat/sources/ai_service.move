module onchain_ai_chat::ai_service {
    use std::string::{Self, String};
    use std::vector;
    use std::option;
    use moveos_std::object::ObjectID;
    use moveos_std::account;
    use verity::oracles;
    use verity::registry;
    use rooch_framework::gas_coin::RGas;
    use rooch_framework::account_coin_store;
    use onchain_ai_chat::message::{Self, Message};

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

    /// Escape special characters in JSON string content
    fun escape_json_string(content: &String): String {
        let result = vector::empty<u8>();
        let bytes = string::bytes(content);
        let i = 0;
        let len = vector::length(bytes);
        while (i < len) {
            let byte = *vector::borrow(bytes, i);
            if (byte == 0x22) { // double quote "
                vector::append(&mut result, b"\\\"");
            } else if (byte == 0x5c) { // backslash \
                vector::append(&mut result, b"\\\\");
            } else if (byte == 0x08) { // backspace
                vector::append(&mut result, b"\\b");
            } else if (byte == 0x0c) { // form feed
                vector::append(&mut result, b"\\f");
            } else if (byte == 0x0a) { // line feed
                vector::append(&mut result, b"\\n");
            } else if (byte == 0x0d) { // carriage return
                vector::append(&mut result, b"\\r");
            } else if (byte == 0x09) { // tab
                vector::append(&mut result, b"\\t");
            } else {
                vector::push_back(&mut result, byte);
            };
            i = i + 1;
        };
        string::utf8(result)
    }

    fun build_chat_context(content: String, previous_messages: &vector<Message>): String {
        //we use a fixed model for now, gpt-4o
        let body = string::utf8(b"{\"model\": \"gpt-4o\", \"messages\": [");
        
        let i = 0;
        let len = vector::length(previous_messages);
        while (i < len) {
            if (i > 0) {
                string::append(&mut body, string::utf8(b","));
            };
            let msg = vector::borrow(previous_messages, i);
            string::append(&mut body, string::utf8(b"{\"role\": \""));
            string::append(&mut body, if (message::get_type(msg) == message::type_ai()) {
                string::utf8(b"assistant")
            } else {
                string::utf8(b"user")
            });
            string::append(&mut body, string::utf8(b"\", \"content\": \""));
            // Escape message content
            string::append(&mut body, escape_json_string(&message::get_content(msg)));
            string::append(&mut body, string::utf8(b"\"}"));
            i = i + 1;
        };

        // Add current message with escaped content
        if (len > 0) {
            string::append(&mut body, string::utf8(b","));
        };
        string::append(&mut body, string::utf8(b"{\"role\": \"user\", \"content\": \""));
        string::append(&mut body, escape_json_string(&content));
        string::append(&mut body, string::utf8(b"\"}], \"temperature\": 0.7}"));

        body
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
        
        let body = build_chat_context(content, &previous_messages);
        
        let pick = string::utf8(AI_PICK);
        let http_request = oracles::build_request(url, method, headers, body);
        
        let option_min_amount = registry::estimated_cost(ORACLE_ADDRESS, url, string::length(&body), 1024);
        let oracle_fee = DEFAULT_ORACLE_FEE*10;
        let _oracle_fee: u256 = if(option::is_some(&option_min_amount)) {
            option::destroy_some(option_min_amount)*2
        } else {
            DEFAULT_ORACLE_FEE
        };
        
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

    #[test]
    fun test_escape_json_string() {
        let test_str = string::utf8(b"Hello \"world\"\nNew line\tTab");
        let escaped = escape_json_string(&test_str);
        assert!(escaped == string::utf8(b"Hello \\\"world\\\"\\nNew line\\tTab"), 1);
    }

    #[test]
    fun test_build_chat_context() {
        use std::string;

        // Test with empty previous messages
        {
            let messages = vector::empty<Message>();
            let content = string::utf8(b"Hello AI");
            let context = build_chat_context(content, &messages);
            let expected = string::utf8(b"{\"model\": \"gpt-4o\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello AI\"}], \"temperature\": 0.7}");
            assert!(context == expected, 1);
        };

        // Test with one previous message
        {
            let messages = vector::empty<Message>();
            vector::push_back(&mut messages, message::new_message(0, @0x1, string::utf8(b"Hi"), message::type_user()));
            let content = string::utf8(b"How are you?");
            let context = build_chat_context(content, &messages);
            let expected = string::utf8(b"{\"model\": \"gpt-4o\", \"messages\": [{\"role\": \"user\", \"content\": \"Hi\"},{\"role\": \"user\", \"content\": \"How are you?\"}], \"temperature\": 0.7}");
            assert!(context == expected, 2);
        };

        // Test with conversation including AI response
        {
            let messages = vector::empty<Message>();
            vector::push_back(&mut messages, message::new_message(0, @0x1, string::utf8(b"Hi"), message::type_user()));
            vector::push_back(&mut messages, message::new_message(1, @0x2, string::utf8(b"Hello! How can I help?"), message::type_ai()));
            let content = string::utf8(b"What's the weather?");
            let context = build_chat_context(content, &messages);
            let expected = string::utf8(b"{\"model\": \"gpt-4o\", \"messages\": [{\"role\": \"user\", \"content\": \"Hi\"},{\"role\": \"assistant\", \"content\": \"Hello! How can I help?\"},{\"role\": \"user\", \"content\": \"What's the weather?\"}], \"temperature\": 0.7}");
            assert!(context == expected, 3);
        };

        // Test with special characters
        {
            let messages = vector::empty<Message>();
            vector::push_back(&mut messages, message::new_message(0, @0x1, string::utf8(b"Hello \"AI\""), message::type_user()));
            let content = string::utf8(b"New\nline");
            let context = build_chat_context(content, &messages);
            let expected = string::utf8(b"{\"model\": \"gpt-4o\", \"messages\": [{\"role\": \"user\", \"content\": \"Hello \\\"AI\\\"\"},{\"role\": \"user\", \"content\": \"New\\nline\"}], \"temperature\": 0.7}");
            assert!(context == expected, 4);
        };
    }
}