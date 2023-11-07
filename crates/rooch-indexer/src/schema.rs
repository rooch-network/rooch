// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// @generated automatically by Diesel CLI.

diesel::table! {
    events (event_handle_id, event_seq) {
        event_handle_id -> Text,
        event_seq -> Int8,
        type_tag -> Text,
        event_data -> Binary,
        event_index -> Int8,
        tx_hash -> Text,
        tx_order -> Int8,
        sender -> Text,
        created_at -> Int8,
        updated_at -> Int8,
    }
}

diesel::table! {
    posts (id) {
        id -> Integer,
        title -> Text,
        body -> Text,
        published -> Bool,
    }
}

diesel::table! {
    transactions (tx_order) {
        tx_order -> Int8,
        tx_hash -> Text,
        transaction_type -> Text,
        sequence_number -> Int8,
        multichain_id -> Int8,
        multichain_raw_address -> Text,
        sender -> Text,
        action -> Text,
        action_type -> Int2,
        action_raw -> Binary,
        auth_validator_id -> Int8,
        authenticator_payload -> Binary,
        tx_accumulator_root -> Text,
        transaction_raw -> Binary,
        state_root -> Text,
        event_root -> Text,
        gas_used -> Int8,
        status -> Text,
        tx_order_auth_validator_id -> Int8,
        tx_order_authenticator_payload -> Binary,
        created_at -> Int8,
        updated_at -> Int8,
    }
}

diesel::allow_tables_to_appear_in_same_query!(events, posts, transactions,);
