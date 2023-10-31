CREATE TABLE transactions (
    tx_order                            INTEGER      NOT NULL      PRIMARY KEY,
    tx_hash                             VARCHAR      NOT NULL,
    transaction_type                    VARCHAR      NOT NULL,
    sequence_number                     INTEGER      NOT NULL,
    multichain_id                       VARCHAR      NOT NULL,
    multichain_raw_address              BLOB         NOT NULL,
    sender                              VARCHAR      NOT NULL,
    action                              VARCHAR      NOT NULL,
    action_type                         SMALLINT     NOT NULL,
    action_raw                          BLOB         NOT NULL,
    auth_validator_id                   INTEGER      NOT NULL,
    authenticator_payload               BLOB         NOT NULL,
    tx_accumulator_root                 VARCHAR      NOT NULL,
    transaction_raw                     BLOB         NOT NULL,

    state_root                          VARCHAR      NOT NULL,
    event_root                          VARCHAR      NOT NULL,
    gas_used                            INTEGER      NOT NULL,
    status                              VARCHAR      NOT NULL,

    tx_order_auth_validator_id          INTEGER      NOT NULL,
    tx_order_authenticator_payload      BLOB         NOT NULL,

    created_at                          INTEGER      NOT NULL,
    updated_at                          INTEGER      NOT NULL,
    UNIQUE (tx_hash)
);

CREATE INDEX idx_transactions_sender ON transactions (sender);
CREATE INDEX idx_transactions_created_at ON transactions (created_at);