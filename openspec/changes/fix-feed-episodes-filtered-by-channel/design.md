## Context

See `proposal.md - Why` for the motivation. Relevant current state:

- `src/handlers/feed.rs:36` calls `Episode::read_all(&data.pool)` inside `get_feed`, which loads every episode row in the DB regardless of channel. The feed's `<enclosure>` URL is built as `{url}/media/{channel_id}/{yt_id}.mp3` (feed.rs:41) using the requested `channel_id`.
- `src/models/episode.rs:103` already provides `read_episodes_for_channel(pool, channel_id)`, a `WHERE channel_id = $1 ORDER BY published_at DESC` query. It is used by the JSON API (`episodes::read_with_pagination`) and is unused by the feed.
- Media files are stored under `/media/{channel_id}/` on disk (`channels::delete` removes `{FOLDER}/{channel_id}`), so `{url}/media/{channel_id}/{yt_id}.mp3` is the correct enclosure URL once items are filtered.

## Goals / Non-Goals

**Goals:**
- Every feed contains exclusively its own channel's episodes.
- Feed items are newest-first.
- Enclosure URLs resolve to real media for the requesting channel.
- Zero new code paths: reuse the existing `read_episodes_for_channel` query.

**Non-Goals:**
- Pagination or limiting of feed items (the current feed includes all episodes of the channel).
- Changing the feed URL scheme or the media storage layout.
- Changing Basic Auth or any access-control behavior.

## Decisions

### Decision 1: Reuse the existing channel-scoped query instead of filtering in Rust

`get_feed` switches from `Episode::read_all(&data.pool)` to `Episode::read_episodes_for_channel(&data.pool, channel_id)`. The SQL already filters by `channel_id` and orders by `published_at DESC`, so both the filtering and the ordering requirements fall out of the query with no client-side code.

**Why**: the query already exists, is exercised by the JSON API, and matches the required ordering. Filtering in Rust after `read_all` would keep loading unrelated rows and would require sorting code for no benefit.

**Alternative considered**: Filter the `read_all` result in Rust by `episode.channel_id == channel_id`. Rejected: it still loads all episodes from disk and the correct scoped query already exists.

### Decision 2: No change to enclosure URL construction

The `<enclosure>` URL stays `{url}/media/{channel_id}/{yt_id}.mp3`. Once items are filtered by channel, `yt_id` always belongs to the requested channel, so the URL points at a file that exists under `/media/{channel_id}/`.

**Why**: the URL template is already correct given correct data; the bug was the data source, not the template.

**Alternative considered**: Prefixing enclosure URLs with the episode's own `channel_id` instead of the requested one. Rejected: the two are identical after filtering, and using the requested channel keeps the URL consistent with the feed path.

## Risks / Trade-offs

- **[Risk] A feed with no episodes still returns `200` with an empty `<item>` list.** → Accepted: this is valid RSS and matches current behavior for empty channels; no change required.
- **[Trade-off] Enclosure URLs change for episodes that were previously misattributed.** A podcast client that already downloaded a wrong episode from another channel may show a stale item until refresh. → Mitigation: none needed; clients refresh feeds on a schedule and will drop the stale items.

## Migration Plan

1. Apply the one-line change in `src/handlers/feed.rs` (`read_all` → `read_episodes_for_channel`).
2. `cargo build` in the container; redeploy.
3. Verify each feed serves only its channel's episodes (see `tasks.md`).

**Rollback**: revert the single line in `feed.rs`. No DB migration or config change to revert.

## Open Questions

None.
