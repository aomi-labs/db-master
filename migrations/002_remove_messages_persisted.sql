-- Remove messages_persisted column from sessions table
-- This column is no longer needed

ALTER TABLE sessions DROP COLUMN IF EXISTS messages_persisted;
