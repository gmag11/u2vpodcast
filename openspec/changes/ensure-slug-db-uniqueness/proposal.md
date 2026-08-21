## Why

The `channel-slugs` spec promises a `NOT NULL, UNIQUE` slug, but the migration adds `slug TEXT` with no unique index. Uniqueness relies on a non-atomic check-then-insert (`unique_slug` → `slug_exists`), so concurrent creations (or a worker `migrate_slugs` racing an insert) can produce duplicate slugs. Two channels then share `{audios}/{slug}/`, and `DELETE /channels/{slug}/` runs `remove_dir_all` on that shared directory — destroying the second channel's files while its rows survive, leaving feeds pointing at missing audio.

## What Changes

- Add a `UNIQUE` index on `channels.slug` (new migration), after safely deduplicating any existing duplicates.
- Make slug generation resilient to DB-level conflicts: on a unique-violation insert, retry with the next `-N` suffix instead of relying on check-then-insert.
- Protect deletion: only remove the audio directory that demonstrably belongs to the deleted channel (ownership guard), so a foreign directory cannot be wiped.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `channel-slugs`: Enforces slug uniqueness at the database level and adds a deletion ownership guard, closing the race and cross-channel wipe.

## Impact

- New migration (up/down), `src/models/channel.rs` (`unique_slug` conflict handling, delete ownership), `src/handlers/channels.rs` (delete path).
- No feed/API URL changes; slug values stable.
- Regression guard: re-analysis against `docs/bug-review-2026-08-21.md`; migration must be safe on existing data (duplicates deduped before the index), and no new bugs may come from the ownership guard (deleting a channel must still remove its own files).