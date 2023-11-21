CREATE TABLE global_states
(
    object_id          VARCHAR        NOT NULL,
    owner              VARCHAR        NOT NULL,
    flag               SMALLINT       NOT NULL,
    value              VARCHAR        NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL,
    PRIMARY KEY (object_id)
);

CREATE INDEX idx_global_states_owner ON global_states (owner);
CREATE INDEX idx_global_states_created_at ON global_states (created_at);