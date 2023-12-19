CREATE TABLE table_states
(
    table_handle       VARCHAR        NOT NULL,
    key_hex            VARCHAR        NOT NULL,
    value              VARCHAR        NOT NULL,
    value_type         VARCHAR        NOT NULL,
    tx_order           BIGINT         NOT NULL,
    state_index        BIGINT         NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL,
    PRIMARY KEY (table_handle, key_hex),
    UNIQUE (tx_order, state_index)
);

CREATE INDEX idx_table_states_created_at ON table_states (created_at);