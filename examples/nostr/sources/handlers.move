// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Implements NIP-01 for handling client messages.
/// There's no relay as handlers deal with all client messages and redirect them to third party applications using oracle http/https proxies.
module nostr::handlers {
    use moveos_std::simple_map::{Self, SimpleMap};
    use moveos_std::simple_multimap::{Self, SimpleMultiMap};
    use moveos_std::account::{AccountCap};
    use nostr::event::{Self, Event};
    use nostr::inner::{Self, Tags};
    use nostr::requests;
    use verity::oracles;
    use verity::registry;

    // types of messages
    const EVENT_KEY: vector<u8> = b"EVENT";
    const REQ_KEY: vector<u8> = b"REQ";
    const CLOSE_KEY: vector<u8> = b"CLOSE";

    // Error codes from 1000 onward
    const ErrorEventMessage: u64 = 1000;
    const ErrorInvalidPublicKeyOwner: u64 = 1001;

    public fun event_key_string(): String {
        string::utf8(EVENT_KEY)
    }

    public fun req_key_string(): String {
        string::utf8(REQ_KEY)
    }

    public fun close_key_string(): String {
        string::utf8(CLOSE_KEY)
    }

    /// Publish an event to supported NIP relays using an external oracle
    public entry fun publish_event(event_message: &SimpleMap<String, String>) {
        assert!(is_event_message(event_message), ErrorEventMessage);

        // parse the event message to event struct
        let event_key = event_key_string();
        let event_json = simple_map::borrow(event_message, &event_key);
        let event = json::from_json<Event>(event_json);

        // check if the event is in the Move's state and whether signer (address) owns the event
        let event_object_id = object::id<Event>(&event);
        let event_object = object::borrow_object<Event>(event_object_id);
        let object_owner = object::owner<Event>(event_object);
        let rooch_address = inner::derive_rooch_address(&event.pubkey);
        assert!(rooch_address == object_owner, ErrorInvalidPublicKeyOwner);

        // create a http/https event request
        // TODO: use params from user config for user defined relays
        let (id, pubkey, created_at, kind, tags, content, sig) = event::unpack_event(event);
        let id_str = string::utf8(id);
        let pubkey_str = string::utf8(pubkey);
        let sig_str = string::utf8(sig);
        let event_request = requests::new_event_request(id_str, pubkey_str, created_at, kind, tags, content, sig_str);
        let event_request_json = requests::event_request_json(&event_request);

        // build oracles http request
        let url = requests::nostr_oracle_url_string();
        let method = requests::nostr_oracle_method_string();
        let headers = requests::nostr_oracle_headers_string();
        let body = string::uft8(event_request_json);
        let http_request = oracles::build_request(url, method, headers, body);

        // TODO: withdraw fees from connected NWC wallet from application interface
        // oracle fees
        let oracle_address = requests::oracle_address();
        let request_body_length = string::length(&body);
        let max_response_length = requests::max_response_length();
        let option_min_amount = registry::estimated_cost(oracle_address, url, request_body_length, max_response_length);
        let default_oracle_fee = requests::default_oracle_fee();
        let estimated_fee: u256 = if(option::is_some(&option_min_amount)) {
            option::destroy_some(option_min_amount)
        } else {
            default_oracle_fee
        };
        let oracle_fee = u256::max(estimated_fee, default_oracle_fee);

        // user balance
        let oracle_balance = oracles::get_user_balance(rooch_address);
        if(oracle_balance < oracle_fee) {
            let pay_mee = oracle_fee - oracle_balance;
            let gas_balance = account_coin_store::balance<RGas>(rooch_address);
            assert!(gas_balance >= pay_mee, ErrorInsufficientBalance);
            let account_cap = AccountCap {
                addr: rooch_address
            };
            let signer = create_signer_with_account_cap(&mut account_cap);
            oracles::deposit_to_escrow(signer, pay_mee);
        };

        // prepare for oracles request params
        let pick = requests::nostr_path_pick_string();
        let notify_callback = requests::notify_callback_string();

        // proxy the http/https event request to the oracle
        let request_id = oracles::new_request(
            http_request,
            pick,
            oracle_address,
            oracles::with_notify(@nostr, notify_callback)
        );

        // update notification gas allocation for the oracle
        let default_notification_gas = requests::default_notification_gas();
        oracles::update_notification_gas_allocation(from, @nostr, notify_callback, default_notification_gas);
    }

    /// TODO: NIP-01: index the a single alphabet letter tag with the first value returned to be used with tag filter from the client
    fun index_tags(tags: vector<Tags>): SimpleMultiMap<String, String> {
        // create a simple multi map for the single-letter english alphabet letters of tag index
        let alphabet = simple_multimap::new<String, String>();
    }

    fun is_event_message(event_message: &SimpleMap<String, String>): bool {
        let event_key = event_key_string();
        simple_map::contains_key(event_message, &event_key)
    }
}
