CREATE TABLE utxos
(
    id                 VARCHAR        NOT NULL       PRIMARY KEY,
    owner              VARCHAR        NOT NULL,
    tx_order           BIGINT         NOT NULL,
    state_index        BIGINT         NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL,
    UNIQUE (tx_order, state_index)
);

CREATE INDEX idx_object_state_utxos_owner ON utxos (owner, tx_order, state_index);
CREATE INDEX idx_object_state_utxos_updated_at ON utxos (updated_at, tx_order, state_index);