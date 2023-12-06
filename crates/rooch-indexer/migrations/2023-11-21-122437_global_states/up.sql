CREATE TABLE global_states
(
    object_id          VARCHAR        NOT NULL       PRIMARY KEY,
    owner              VARCHAR        NOT NULL,
    flag               SMALLINT       NOT NULL,
    value              VARCHAR        NOT NULL,
    value_type         VARCHAR        NOT NULL,
    key_type           VARCHAR        NOT NULL,
    size               BIGINT         NOT NULL,
    tx_order           BIGINT         NOT NULL,
    state_index        BIGINT         NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL,
    UNIQUE (tx_order, state_index)
);

CREATE INDEX idx_global_states_owner ON global_states (owner);
CREATE INDEX idx_global_states_value_type_and_owner ON global_states (value_type, owner);
CREATE INDEX idx_global_states_created_at ON global_states (created_at);