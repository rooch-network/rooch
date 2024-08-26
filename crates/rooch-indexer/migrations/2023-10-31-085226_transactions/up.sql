CREATE TABLE transactions (
    tx_order                            BIGINT       NOT NULL      PRIMARY KEY,
    tx_hash                             VARCHAR      NOT NULL,
    sequence_number                     BIGINT       NOT NULL,
    sender                              VARCHAR      NOT NULL,
    action_type                         SMALLINT     NOT NULL,
    auth_validator_id                   BIGINT       NOT NULL,
    gas_used                            BIGINT       NOT NULL,
    status                              VARCHAR      NOT NULL,
    created_at                          BIGINT       NOT NULL,
    UNIQUE (tx_hash)
);

CREATE INDEX idx_transactions_sender ON transactions (sender, tx_order);
--CREATE INDEX idx_transactions_tx_hash ON transactions (tx_hash, tx_order);
CREATE INDEX idx_transactions_created_at ON transactions (created_at, tx_order);