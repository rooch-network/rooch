// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// @generated automatically by Diesel CLI.

diesel::table! {
    transactions (tx_order) {
        tx_order -> Integer,
        tx_hash -> Text,
        transaction_type -> Text,
        sequence_number -> Integer,
        multichain_id -> Text,
        multichain_raw_address -> Blob,
        sender -> Text,
        action -> Text,
        action_type -> SmallInt,
        action_raw -> Blob,
        auth_validator_id -> Integer,
        authenticator_payload -> Blob,
        tx_accumulator_root -> Text,
        transaction_raw -> Blob,

        state_root -> Text,
        event_root -> Text,
        gas_used -> Integer,
        status -> Text,

        tx_order_auth_validator_id -> Integer,
        tx_order_authenticator_payload -> Blob,

        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::table! {
    events (event_handle_id, event_seq) {
        event_handle_id -> Text,
        event_seq -> Integer,
        type_tag -> Text,
        event_data -> Blob,
        event_index -> Integer,

        tx_hash -> Text,
        tx_order -> Integer,
        sender -> Text,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(transactions, events,);
