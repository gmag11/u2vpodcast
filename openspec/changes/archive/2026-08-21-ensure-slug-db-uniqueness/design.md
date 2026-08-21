## Context

Migration `20260810000001_add_slug` only adds the column. `unique_slug`/`slug_exists` (`src/models/channel.rs:148-175`) implement check-then-insert; nothing else guards uniqueness. `DELETE` (handlers/channels.rs) calls `remove_dir_all` on `{audios}/{slug}` unconditionally.

## Goals / Non-Goals

**Goals:**
- Slug uniqueness at the DB layer, race-proof.
- Delete cannot destroy another channel's audio.

**Non-Goals:**
- No change to slug format rules, URL scheme, or the startup backfill logic (still idempotent).
- No UI changes.

## Decisions

- **New migration adding `CREATE UNIQUE INDEX` on `channels.slug`**, preceded by a deterministic dedupe pass that appends `-N` suffixes to existing duplicates (same suffix scheme as creation, so results are stable across runs). Down migration drops the index.
- **Conflict retry instead of relying on check-then-insert:** wrap the insert to detect the unique-violation error and regenerate with the next suffix; `slug_exists` stays as a fast path but is no longer the only protection.
- **Ownership guard on delete:** before `remove_dir_all`, check that no other channel row references the same slug path (defense-in-depth; after the index it can only be empty-slug or stale cases, in which case we log and skip).

## Risks / Trade-offs

- [Migration on a dirty DB with many duplicates] → Dedupe pass before index creation; deterministic suffixes; reversible down.
- [Rename flows (bug #6) could interact with slugs] → Slug is immutable by spec; this change does not touch rename.

## Migration Plan

1. Ship dedupe+index migration.
2. Ship code retry/guard.
3. Rollback: down migration drops the index; code still safe under check-then-insert (previous behavior).

## Open Questions

Should the delete ownership guard also handle a directory that exists with no matching rows? Implementation should log-and-skip for that case (orphan cleanup is out of scope).