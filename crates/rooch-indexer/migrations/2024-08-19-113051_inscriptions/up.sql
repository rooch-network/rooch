CREATE TABLE inscriptions
(
    id                 VARCHAR        NOT NULL       PRIMARY KEY,
    owner              VARCHAR        NOT NULL,
--    flag               SMALLINT       NOT NULL,
--    state_root         VARCHAR        NOT NULL,
--    size               BIGINT         NOT NULL,
--    object_type        VARCHAR        NOT NULL,
    tx_order           BIGINT         NOT NULL,
    state_index        BIGINT         NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL,
    UNIQUE (tx_order, state_index)
);

--CREATE INDEX idx_object_state_inscriptions_owner_and_object_type ON inscriptions (owner, object_type);
--CREATE INDEX idx_object_state_inscriptions_object_type ON inscriptions (object_type);
CREATE INDEX idx_object_state_inscriptions_owner ON inscriptions (owner, tx_order, state_index);