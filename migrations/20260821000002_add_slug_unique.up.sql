-- Backfill: deduplicate slug values shared by more than one channel before the
-- UNIQUE index can be created. Duplicate rows (all but the first one, i.e. the
-- one with the lowest id in each duplicate group) get a deterministic suffix
-- `-dup-<id>`; ids are unique so the suffix cannot collide with another row's
-- id, and `-dup-` is outside the creation-time `-N` numbering scheme.
UPDATE channels
SET slug = slug || '-dup-' || id
WHERE slug IS NOT NULL AND slug <> ''
  AND (SELECT COUNT(*) FROM channels AS other WHERE other.slug = channels.slug) > 1;

CREATE UNIQUE INDEX idx_channels_slug_unique ON channels(slug);