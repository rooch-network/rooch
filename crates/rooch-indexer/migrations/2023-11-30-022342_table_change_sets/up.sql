CREATE TABLE table_change_sets
(
    tx_order             BIGINT         NOT NULL,
    table_handle_index   BIGINT         NOT NULL,
    table_handle         VARCHAR        NOT NULL,
    table_change_set     VARCHAR        NOT NULL,
    created_at           BIGINT         NOT NULL,
    PRIMARY KEY (tx_order, table_handle_index),
    UNIQUE (tx_order, table_handle)
);

CREATE INDEX idx_table_change_sets_table_handle ON table_change_sets (table_handle);
CREATE INDEX idx_table_change_sets_created_at ON table_change_sets (created_at);