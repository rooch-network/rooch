-- down.sql

-- Drop the newly created indexes
DROP INDEX IF EXISTS idx_object_states_object_type;
DROP INDEX IF EXISTS idx_object_states_owner_and_object_type;

DROP INDEX IF EXISTS idx_events_event_type;