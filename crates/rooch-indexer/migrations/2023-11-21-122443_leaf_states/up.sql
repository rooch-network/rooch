CREATE TABLE table_states
(
    id                 VARCHAR        NOT NULL      PRIMARY KEY,
    table_handle       VARCHAR        NOT NULL,
    key_hex            VARCHAR        NOT NULL,
    value              VARCHAR        NOT NULL,
    value_type         VARCHAR        NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL,
    UNIQUE (table_handle, key_hex)
);

CREATE INDEX idx_table_states_object_id ON table_states (table_handle);
CREATE INDEX idx_table_states_created_at ON table_states (created_at);