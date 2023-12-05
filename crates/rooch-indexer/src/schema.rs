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
    global_states (object_id) {
        object_id -> Text,
        owner -> Text,
        flag -> SmallInt,
        value -> Text,
        object_type -> Text,
        key_type -> Text,
        size -> BigInt,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    table_states (id) {
        id -> Text,
        table_handle -> Text,
        key_hex -> Text,
        value -> Text,
        value_type -> Text,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    table_change_sets (tx_order, table_handle_index) {
        tx_order -> BigInt,
        table_handle_index -> BigInt,
        table_handle -> Text,
        table_change_set -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    transactions (tx_order) {
        tx_order -> BigInt,
        tx_hash -> Text,
        transaction_type -> Text,
        sequence_number -> BigInt,
        multichain_id -> BigInt,
        multichain_address -> Text,
        multichain_original_address -> Text,
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
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    events,
    global_states,
    table_states,
    table_change_sets,
    transactions,
);
