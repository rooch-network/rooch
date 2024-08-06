-- up.sql

-- Drop existing indexes
DROP INDEX IF EXISTS idx_object_states_owner;
DROP INDEX IF EXISTS idx_object_states_object_type_and_owner;

DROP INDEX IF EXISTS idx_events_tx_order;
-- Add more DROP INDEX statements as needed

-- Create indexes
CREATE INDEX idx_object_states_object_type ON object_states (object_type);
CREATE INDEX idx_object_states_owner_and_object_type ON object_states (owner, object_type);

CREATE INDEX idx_events_event_type ON events (event_type);
-- Add more CREATE INDEX statements as needed


-- Reindex indexes
REINDEX object_states;
REINDEX events;
-- Add more REINDEX INDEX statements as needed

