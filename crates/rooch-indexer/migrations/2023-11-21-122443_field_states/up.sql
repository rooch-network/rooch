CREATE TABLE field_states
(
    object_id       VARCHAR        NOT NULL,
    key_hex            VARCHAR        NOT NULL,
    key_str            VARCHAR        NOT NULL,
    key_type           VARCHAR        NOT NULL,
    value_type         VARCHAR        NOT NULL,
    tx_order           BIGINT         NOT NULL,
    state_index        BIGINT         NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL,
    PRIMARY KEY (object_id, key_hex),
    UNIQUE (tx_order, state_index)
);

CREATE INDEX idx_field_states_created_at ON field_states (created_at);