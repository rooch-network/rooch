CREATE TABLE state_change_sets
(
    tx_order           BIGINT         NOT NULL         PRIMARY KEY,
    state_change_set   VARCHAR        NOT NULL,
    created_at         BIGINT         NOT NULL
);

CREATE INDEX idx_state_change_sets_created_at ON state_change_sets (created_at);