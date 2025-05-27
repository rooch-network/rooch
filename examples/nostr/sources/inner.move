// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

/// Internal data structures
module nostr::inner {
    use std::vector;
    use std::string::{Self, String};
    use std::option::{Self, Option};
    use moveos_std::bcs;
    use moveos_std::string_utils;
    use rooch_framework::bitcoin_address::{Self, BitcoinAddress};

    // Name of the tag of the event
    const EVENT_TAG_KEY: vector<u8> = b"e";
    const USER_TAG_KEY: vector<u8> = b"p";
    const ADDRESSABLE_REPLACEABLE_TAG_KEY: vector<u8> = b"a";

    // Punctuation marks
    const COLON: vector<u8> = b":";

    // Hex literals for kind max value
    const KIND_UPPER_VALUE: u16 = 0xFFFF;

    // Content filter in NIP-01
    const DOUBLEQUOTE: u8 = 34; // 0x22, \", Double Quote
    const BACKSLASH: u8 = 92; // 0x5C, \\, Backslash

    // Error codes starting from 1000
    const ErrorMalformedOtherEventId: u64 = 1000;
    const ErrorMalformedPublicKey: u64 = 1001;
    const ErrorKindOutOfRange: u64 = 1002;
    const ErrorEmptyTagString: u64 = 1003;

    #[data_struct]
    /// Tags
    struct Tags has copy, drop {
        // For referring to an event
        event: Option<EventTag>,
        // For another user
        user: Option<UserTag>,
        // For addressable or replaceable events
        addressable_replaceable: Option<AddressableReplaceableTag>,
        // For other arrays of non-null strings
        strings_list: Option<StringsListTag>
    }

    #[data_struct]
    /// EventTag with `e` key or name
    struct EventTag has copy, drop {
        // id as value
        id: vector<u8>,
        url: Option<String>,
        pubkey: Option<vector<u8>>
    }

    #[data_struct]
    /// UserTag with `p` key or name
    struct UserTag has copy, drop {
        // pubkey as value
        pubkey: vector<u8>,
        url: Option<String>
    }

    #[data_struct]
    /// AddressableReplaceableTag with `a` key or name
    struct AddressableReplaceableTag has copy, drop {
        // kind:pubkey:d as value
        kind: u16,
        pubkey: vector<u8>,
        d: Option<String>,
        url: Option<String>
    }

    #[data_struct]
    /// StringsListTag with non-empty array value of strings
    struct StringsListTag has copy, drop {
        // the first and second elements of this string list are key or name and value
        strings: vector<String>
    }

    public fun event_tag_key_string(): String {
        string::utf8(EVENT_TAG_KEY)
    }

    public fun user_tag_key_string(): String {
        string::utf8(USER_TAG_KEY)
    }

    public fun addressable_replaceable_tag_key_string(): String {
        string::utf8(ADDRESSABLE_REPLACEABLE_TAG_KEY)
    }

    public fun colon_string(): String {
        string::utf8(COLON)
    }

    public fun doublequote(): u8 {
        DOUBLEQUOTE
    }

    public fun backslash(): u8 {
        BACKSLASH
    }

    /// derive a bitcoin taproot address from a x-only public key
    public fun derive_bitcoin_taproot_address(x_only_public_key: vector<u8>): BitcoinAddress {
        // derive a bitcoin taproot address from the x only public key
        let bitcoin_taproot_address = bitcoin_address::derive_bitcoin_taproot_address_from_pubkey(&x_only_public_key);
        bitcoin_taproot_address
    }

    /// derive a rooch address from a bitcoin taproot address from a x-only public key
    public fun derive_rooch_address(x_only_public_key: vector<u8>): address {
        let bitcoin_taproot_address = derive_bitcoin_taproot_address(x_only_public_key);
        // derive a rooch address from the bitcoin taproot address
        let rooch_address = bitcoin_address::to_rooch_address(&bitcoin_taproot_address);
        rooch_address
    }

    /// build string tags to inner struct tags
    public fun build_tags(tags_str: vector<vector<String>>): vector<Tags> {
        // init tags list
        let tags_list = vector::empty<Tags>();
        // perform build
        let i = 0;
        let tags_str_len = vector::length(&tags_str);
        while (i < tags_str_len) {
            // init empty string list
            let string_list = vector::empty<String>();
            let tag_str_list = vector::borrow(&tags_str, i);
            let o = 0;
            let tag_str_list_len = vector::length(tag_str_list);
            while (o < tag_str_list_len) {
                let tag_str = vector::borrow(tag_str_list, o);
                assert!(!string::is_empty(tag_str), ErrorEmptyTagString);
                // take special circumstance for the NIP-01 defined tag keys
                if (o == 0) {
                    let tag_value_index = o + 1;
                    let tag_value = vector::borrow(tag_str_list, tag_value_index);
                    // EVENT_TAG_KEY
                    if (*tag_str == event_tag_key_string()) {
                        // get the id of another Event
                        let id = bcs::to_bytes(tag_value);
                        assert!(vector::length(&id) == 32, ErrorMalformedOtherEventId);
                        // get the url of recommended relay if it exists
                        let url_option = option::none<String>();
                        if (tag_str_list_len == 3) {
                            let index = o + 2;
                            let str = vector::borrow(tag_str_list, index);
                            option::fill<String>(&mut url_option, *str);
                        };
                        // get the author's public key if it exists
                        let pubkey_option = option::none<vector<u8>>();
                        if (tag_str_list_len == 4) {
                            let index = o + 3;
                            let pubkey = vector::borrow(tag_str_list, index);
                            let pubkey_bytes = string::bytes(pubkey);
                            option::fill<vector<u8>>(&mut pubkey_option, *pubkey_bytes);
                        };
                        let event_tag = EventTag {
                            id,
                            url: url_option,
                            pubkey: pubkey_option
                        };
                        let tags = Tags {
                            event: option::some(event_tag),
                            user: option::none<UserTag>(),
                            addressable_replaceable: option::none<AddressableReplaceableTag>(),
                            strings_list: option::none<StringsListTag>()
                        };
                        vector::push_back(&mut tags_list, tags);
                        i = i + 1;
                        break
                    };
                    // USER_TAG_KEY
                    if (*tag_str == user_tag_key_string()) {
                        // get the public key
                        let pubkey = bcs::to_bytes(tag_value);
                        assert!(vector::length(&pubkey) == 32, ErrorMalformedPublicKey);
                        // get the url of recommended relay if it exists
                        let url_option = option::none<String>();
                        if (tag_str_list_len == 3) {
                            let index = o + 2;
                            let url = vector::borrow(tag_str_list, index);
                            option::fill<String>(&mut url_option, *url);
                        };
                        let user_tag = UserTag {
                            pubkey,
                            url: url_option,
                        };
                        let tags = Tags {
                            event: option::none<EventTag>(),
                            user: option::some(user_tag),
                            addressable_replaceable: option::none<AddressableReplaceableTag>(),
                            strings_list: option::none<StringsListTag>()
                        };
                        vector::push_back(&mut tags_list, tags);
                        i = i + 1;
                        break
                    };
                    // ADDRESSABLE_REPLACEABLE_TAG_KEY
                    if (*tag_str == addressable_replaceable_tag_key_string()) {
                        // get first colon position
                        let first_occur_colon_pos = string::index_of(tag_value, &colon_string());
                        // kind of the string
                        let kind = string::sub_string(tag_value, 0, first_occur_colon_pos);
                        let kind_value = string_utils::parse_u16(&kind);
                        assert!(kind_value <= KIND_UPPER_VALUE, ErrorKindOutOfRange);
                        // get the length of the string
                        let tag_value_len = string::length(tag_value);
                        // get remaining values of the string, ignore the first colon
                        let remain_str = string::sub_string(tag_value, first_occur_colon_pos + 1, tag_value_len);
                        // get second colon position
                        let second_occur_colon_pos = string::index_of(&remain_str, &colon_string());
                        // public key of the string
                        let pubkey = string::sub_string(&remain_str, 0, second_occur_colon_pos);
                        let pubkey_bytes = string::bytes(&pubkey);
                        let pubkey_len = vector::length(pubkey_bytes);
                        assert!(pubkey_len == 32, ErrorMalformedPublicKey);
                        // handle d tag value
                        let d_tag_option = option::none<String>();
                        if (second_occur_colon_pos + 1 <= tag_value_len) {
                            let d_tag = string::sub_string(&remain_str, second_occur_colon_pos + 1, tag_value_len);
                            option::fill<String>(&mut d_tag_option, d_tag);
                        };
                        // get the url of recommended relay if it exists
                        let url_option = option::none<String>();
                        if (tag_str_list_len == 3) {
                            let index = o + 2;
                            let url = vector::borrow(tag_str_list, index);
                            option::fill<String>(&mut url_option, *url);
                        };
                        let addressable_replaceable_tag = AddressableReplaceableTag {
                            kind: kind_value,
                            pubkey: *pubkey_bytes,
                            d: d_tag_option,
                            url: url_option,
                        };
                        let tags = Tags {
                            event: option::none<EventTag>(),
                            user: option::none<UserTag>(),
                            addressable_replaceable: option::some(addressable_replaceable_tag),
                            strings_list: option::none<StringsListTag>()
                        };
                        vector::push_back(&mut tags_list, tags);
                        i = i + 1;
                        break
                    };
                };
                // proceed with normal arbitrary tags
                vector::push_back(&mut string_list, *tag_str);
                o = o + 1;
            };
            // submit a list of strings to the tags and push to the tags list
            let strings_list_tag = StringsListTag {
                strings: string_list
            };
            let tags = Tags {
                event: option::none<EventTag>(),
                user: option::none<UserTag>(),
                addressable_replaceable: option::none<AddressableReplaceableTag>(),
                strings_list: option::some(strings_list_tag)
            };
            vector::push_back(&mut tags_list, tags);
            i = i + 1;
        };
        tags_list
    }

    /// flatten inner struct tags into non-null string tags
    public fun flatten_tags(tags: vector<Tags>): vector<vector<String>> {
        // init tags string list
        let tags_string_list = vector::empty<vector<String>>();
        // init tags string
        let tags_string = vector::empty<String>();

        let i = 0;
        let tags_len = vector::length(&tags);
        while (i < tags_len) {
            let inner_tags = vector::borrow_mut(&mut tags, i);
            // event tag
            if (option::is_some(&inner_tags.event)) {
                let event = option::extract(&mut inner_tags.event);
                // key e for the event tag
                let event_tag_key = event_tag_key_string();
                vector::push_back(&mut tags_string, event_tag_key);
                // event id
                let id = string::utf8(event.id);
                vector::push_back(&mut tags_string, id);
                // event url
                if (option::is_some(&event.url)) {
                    let url = option::extract(&mut event.url);
                    vector::push_back(&mut tags_string, url);
                };
                // event pubkey
                if (option::is_some(&event.pubkey)) {
                    let pubkey = option::extract(&mut event.pubkey);
                    let pubkey_str = string::utf8(pubkey);
                    vector::push_back(&mut tags_string, pubkey_str);
                };
                vector::push_back(&mut tags_string_list, tags_string);
            };
            // user tag
            if (option::is_some(&inner_tags.user)) {
                let user = option::extract(&mut inner_tags.user);
                // key p for the user tag
                let user_tag_key = user_tag_key_string();
                vector::push_back(&mut tags_string, user_tag_key);
                // user pubkey
                let pubkey = string::utf8(user.pubkey);
                vector::push_back(&mut tags_string, pubkey);
                // user url
                if (option::is_some(&user.url)) {
                    let url = option::extract(&mut user.url);
                    vector::push_back(&mut tags_string, url);
                };
                vector::push_back(&mut tags_string_list, tags_string);
            };
            // addressable replaceable tag
            if (option::is_some(&inner_tags.addressable_replaceable)) {
                let addressable_replaceable = option::extract(&mut inner_tags.addressable_replaceable);
                // key a for the addressable replaceable tag
                let addressable_replaceable_tag_key = addressable_replaceable_tag_key_string();
                vector::push_back(&mut tags_string, addressable_replaceable_tag_key);
                // init kind:pubkey:d string
                let kind_pubkey_d_str = string::utf8(vector::empty<u8>());
                // addressable replaceable kind
                let kind = string_utils::to_string_u16(addressable_replaceable.kind);
                string::append(&mut kind_pubkey_d_str, kind);
                string::append(&mut kind_pubkey_d_str, colon_string());
                // addressable replaceable pubkey
                let pubkey = string::utf8(addressable_replaceable.pubkey);
                string::append(&mut kind_pubkey_d_str, pubkey);
                string::append(&mut kind_pubkey_d_str, colon_string());
                // addressable replaceable d
                if (option::is_some(&addressable_replaceable.d)) {
                    let d = option::extract(&mut addressable_replaceable.d);
                    string::append(&mut kind_pubkey_d_str, d);
                };
                vector::push_back(&mut tags_string, kind_pubkey_d_str);
                // addressable replaceable url
                if (option::is_some(&addressable_replaceable.url)) {
                    let url = option::extract(&mut addressable_replaceable.url);
                    vector::push_back(&mut tags_string, url);
                };
                vector::push_back(&mut tags_string_list, tags_string);
            };
            // strings list tag
            if (option::is_some(&inner_tags.strings_list)) {
                let strings_list = option::extract(&mut inner_tags.strings_list);
                // strings list strings
                vector::push_back(&mut tags_string_list, strings_list.strings);
            };
            i = i + 1;
        };
        tags_string_list
    }
}
