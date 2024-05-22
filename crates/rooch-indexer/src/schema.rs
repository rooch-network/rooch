// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// @generated automatically by Diesel CLI.

diesel::table! {
    events (event_index, tx_order) {
        event_handle_id -> Text,
        event_seq -> BigInt,
        event_type -> Text,
        event_data -> Binary,
        event_index -> BigInt,
        tx_hash -> Text,
        tx_order -> BigInt,
        sender -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    object_states (object_id) {
        object_id -> Text,
        owner -> Text,
        flag -> SmallInt,
        state_root -> Text,
        size -> BigInt,
        object_type -> Text,
        tx_order -> BigInt,
        state_index -> BigInt,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    field_states (object_id, key_hex) {
        object_id -> Text,
        key_hex -> Text,
        key_type -> Text,
        value_type -> Text,
        tx_order -> BigInt,
        state_index -> BigInt,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    transactions (tx_order) {
        tx_order -> BigInt,
        tx_hash -> Text,
        sequence_number -> BigInt,
        sender -> Text,
        action -> Text,
        action_type -> SmallInt,
        action_raw -> Binary,
        auth_validator_id -> BigInt,
        authenticator_payload -> Binary,
        tx_accumulator_root -> Text,
        transaction_raw -> Binary,
        state_root -> Text,
        size -> BigInt,
        event_root -> Text,
        gas_used -> BigInt,
        status -> Text,
        tx_order_auth_validator_id -> BigInt,
        tx_order_authenticator_payload -> Binary,
        created_at -> BigInt,
    }
}

diesel::allow_tables_to_appear_in_same_query!(events, object_states, field_states, transactions,);
