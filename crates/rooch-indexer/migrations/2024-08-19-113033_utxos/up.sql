CREATE TABLE utxos
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

--CREATE INDEX idx_object_state_utxos_owner_and_object_type ON utxos (owner, object_type, tx_order, state_index);
--CREATE INDEX idx_object_state_utxos_object_type ON utxos (object_type, tx_order, state_index);
CREATE INDEX idx_object_state_utxos_owner ON utxos (owner, tx_order, state_index);