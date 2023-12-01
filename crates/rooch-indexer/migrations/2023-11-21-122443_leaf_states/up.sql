CREATE TABLE leaf_states
(
    id                 VARCHAR        NOT NULL      PRIMARY KEY,
    object_id          VARCHAR        NOT NULL,
    key_hex            VARCHAR        NOT NULL,
    value              VARCHAR        NOT NULL,
    value_type         VARCHAR        NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL,
    UNIQUE (object_id, key_hex)
);

CREATE INDEX idx_leaf_states_object_id ON leaf_states (object_id);
CREATE INDEX idx_leaf_states_created_at ON leaf_states (created_at);