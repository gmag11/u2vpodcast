ALTER TABLE sponsorblock_cache ADD COLUMN processing_hash TEXT;
UPDATE sponsorblock_cache
SET processing_hash = snapshot_hash
WHERE processing_hash IS NULL;
