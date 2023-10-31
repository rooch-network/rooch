// @generated automatically by Diesel CLI.

diesel::table! {
    events (event_handle_id, event_seq) {
        event_handle_id -> Text,
        event_seq -> Integer,
        type_tag -> Text,
        event_data -> Binary,
        event_index -> Integer,
        tx_hash -> Text,
        tx_order -> Integer,
        sender -> Text,
        created_at -> Integer,
        updated_at -> Integer,
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
        tx_order -> Integer,
        tx_hash -> Text,
        transaction_type -> Text,
        sequence_number -> Integer,
        multichain_id -> Text,
        multichain_raw_address -> Binary,
        sender -> Text,
        action -> Text,
        action_type -> SmallInt,
        action_raw -> Binary,
        auth_validator_id -> Integer,
        authenticator_payload -> Binary,
        tx_accumulator_root -> Text,
        transaction_raw -> Binary,
        state_root -> Text,
        event_root -> Text,
        gas_used -> Integer,
        status -> Text,
        tx_order_auth_validator_id -> Integer,
        tx_order_authenticator_payload -> Binary,
        created_at -> Integer,
        updated_at -> Integer,
    }
}

diesel::allow_tables_to_appear_in_same_query!(events, posts, transactions,);
