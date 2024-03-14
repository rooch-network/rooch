CREATE TABLE transactions (
    tx_order                            BIGINT       NOT NULL      PRIMARY KEY,
    tx_hash                             VARCHAR      NOT NULL,
    transaction_type                    VARCHAR      NOT NULL,
    sequence_number                     BIGINT       NOT NULL,
    multichain_id                       BIGINT       NOT NULL,
    multichain_address                  VARCHAR      NOT NULL,
    multichain_original_address         VARCHAR      NOT NULL,
    sender                              VARCHAR      NOT NULL,
    action                              VARCHAR      NOT NULL,
    action_type                         SMALLINT     NOT NULL,
    action_raw                          BLOB         NOT NULL,
    auth_validator_id                   BIGINT       NOT NULL,
    authenticator_payload               BLOB         NOT NULL,
    tx_accumulator_root                 VARCHAR      NOT NULL,
    transaction_raw                     BLOB         NOT NULL,

    state_root                          VARCHAR      NOT NULL,
    size                                BIGINT       NOT NULL,
    event_root                          VARCHAR      NOT NULL,
    gas_used                            BIGINT       NOT NULL,
    status                              VARCHAR      NOT NULL,

    tx_order_auth_validator_id          BIGINT       NOT NULL,
    tx_order_authenticator_payload      BLOB         NOT NULL,

    created_at                          BIGINT       NOT NULL,
    UNIQUE (tx_hash)
);

CREATE INDEX idx_transactions_sender ON transactions (sender);
CREATE INDEX idx_transactions_created_at ON transactions (created_at);