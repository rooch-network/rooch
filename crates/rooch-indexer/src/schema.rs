// Copyright (c) RoochNetwork
// SPDX-License-Identifier: Apache-2.0

// @generated automatically by Diesel CLI.

diesel::table! {
    events (event_index, tx_order) {
        event_handle_id -> Text,
        event_seq -> BigInt,
        event_type -> Text,
        event_index -> BigInt,
        tx_hash -> Text,
        tx_order -> BigInt,
        sender -> Text,
        created_at -> BigInt,
    }
}

diesel::table! {
    object_states (id) {
        id -> Text,
        owner -> Text,
        object_type -> Text,
        tx_order -> BigInt,
        state_index -> BigInt,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    utxos (id) {
        id -> Text,
        owner -> Text,
        tx_order -> BigInt,
        state_index -> BigInt,
        created_at -> BigInt,
        updated_at -> BigInt,
    }
}

diesel::table! {
    inscriptions (id) {
        id -> Text,
        owner -> Text,
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
        action_type -> SmallInt,
        auth_validator_id -> BigInt,
        gas_used -> BigInt,
        status -> Text,
        created_at -> BigInt,
    }
}

diesel::allow_tables_to_appear_in_same_query!(
    events,
    object_states,
    utxos,
    inscriptions,
    transactions,
);
