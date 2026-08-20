ALTER TABLE channels ADD COLUMN last_sync_at DATETIME;
ALTER TABLE channels ADD COLUMN last_sync_ok BOOLEAN;
ALTER TABLE channels ADD COLUMN last_sync_error TEXT;
