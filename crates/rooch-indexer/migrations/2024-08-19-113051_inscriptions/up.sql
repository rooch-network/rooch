CREATE TABLE inscriptions
(
    id                 VARCHAR        NOT NULL       PRIMARY KEY,
    owner              VARCHAR        NOT NULL,
    tx_order           BIGINT         NOT NULL,
    state_index        BIGINT         NOT NULL,
    created_at         BIGINT         NOT NULL,
    updated_at         BIGINT         NOT NULL,
    UNIQUE (tx_order, state_index)
);

CREATE INDEX idx_object_state_inscriptions_owner ON inscriptions (owner, tx_order, state_index);
CREATE INDEX idx_object_state_inscriptions_updated_at ON inscriptions (updated_at, tx_order, state_index);