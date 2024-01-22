CREATE TABLE global_states
(
    object_id          VARCHAR        NOT NULL       PRIMARY KEY,
    owner              VARCHAR        NOT NULL,
    flag               SMALLINT       NOT NULL,
    value              VARCHAR        NOT NULL,
    state_root         VARCHAR        NOT NULL,
    size               BIGINT         NOT NULL,
    object_type        VARCHAR        NOT NULL,
    tx_order           BIGINT         NOT NULL,
    state_index        BIGINT         NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL,
    UNIQUE (tx_order, state_index)
);

CREATE INDEX idx_global_states_owner ON global_states (owner);
CREATE INDEX idx_global_states_object_type_and_owner ON global_states (object_type, owner);
CREATE INDEX idx_global_states_created_at ON global_states (created_at);