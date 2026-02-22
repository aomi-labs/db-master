-- Remove unused columns from sessions table

ALTER TABLE sessions DROP COLUMN IF EXISTS messages_persisted;
ALTER TABLE sessions DROP COLUMN IF EXISTS pending_transaction;
