// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// @generated automatically by Diesel CLI.

diesel::table! {
    events (event_handle_id, event_seq) {
        event_handle_id -> Text,
        event_seq -> BigInt,
        event_type -> Text,
        event_data -> Binary,
        event_index -> BigInt,
        tx_hash -> Text,
        tx_order -> BigInt,
        sender -> Text,
        created_at -> BigInt,
        updated_at -> BigInt,
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
        tx_order -> BigInt,
        tx_hash -> Text,
        transaction_type -> Text,
        sequence_number -> BigInt,
        multichain_id -> BigInt,
        multichain_raw_address -> Text,
        sender -> Text,
        action -> Text,
        action_type -> SmallInt,
        action_raw -> Binary,
        auth_validator_id -> BigInt,
        authenticator_payload -> Binary,
        tx_accumulator_root -> Text,
        transaction_raw -> Binary,
        state_root -> Text,
        event_root -> Text,
        gas_used -> BigInt,
        status -> Text,
        tx_order_auth_validator_id -> BigInt,
        tx_order_authenticator_payload -> Binary,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    events,
    posts,
    transactions,
);
