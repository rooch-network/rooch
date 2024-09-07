CREATE TABLE events
(
    event_handle_id    VARCHAR      NOT NULL,
    event_seq          BIGINT       NOT NULL,
    event_type         VARCHAR      NOT NULL,
    event_index        BIGINT       NOT NULL,

    tx_hash            VARCHAR      NOT NULL,
    tx_order           BIGINT       NOT NULL,
    sender             VARCHAR      NOT NULL,
    created_at         BIGINT       NOT NULL,
    -- Constraints
    PRIMARY KEY (tx_order, event_index),
    UNIQUE (event_handle_id, event_seq)
);


CREATE INDEX idx_events_sender_and_event_type ON events (sender, event_type, tx_order, event_index);
CREATE INDEX idx_events_event_type ON events (event_type, tx_order, event_index);
CREATE INDEX idx_events_sender_and_event_handle ON events (sender, event_handle_id, tx_order, event_index);
CREATE INDEX idx_events_event_handle ON events (event_handle_id, tx_order, event_index);
CREATE INDEX idx_events_sender ON events (sender, tx_order, event_index);
CREATE INDEX idx_events_tx_hash ON events (tx_hash, tx_order, event_index);
CREATE INDEX idx_events_created_at ON events (created_at, tx_order, event_index);