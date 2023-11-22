CREATE TABLE leaf_states
(
    object_id          VARCHAR        NOT NULL      PRIMARY KEY,
    key_hash           VARCHAR        NOT NULL,
    value              VARCHAR        NOT NULL,
    value_type         VARCHAR        NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL
);

CREATE INDEX idx_leaf_states_key ON leaf_states (key_hash);
CREATE INDEX idx_leaf_states_created_at ON leaf_states (created_at);