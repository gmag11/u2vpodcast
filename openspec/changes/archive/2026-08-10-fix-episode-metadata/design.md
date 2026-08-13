## Context

See `proposal.md - Why` for the motivation. Relevant current state:

- `src/utils/worker.rs:161-168` (`parse_date` function) parses yt-dlp's `upload_date` (`YYYYMMDD`) into `DateTime<Utc>` with time fixed at midnight UTC. All episodes from the same day get the same `published_at`, so ordering within a day is arbitrary.
- yt-dlp's `--dump-json` output includes a `timestamp` field (Unix epoch in seconds) that gives the exact upload time with second precision. The current `YtVideo` struct in `src/models/ytdlp.rs:15-25` does not capture it.
- `src/handlers/feed.rs:59` sets `<pubDate>` via `episode.published_at.to_string()`, producing `"YYYY-MM-DD HH:MM:SS UTC"` — not valid RFC 2822.
- `src/handlers/feed.rs:56` sets `<description>` to `episode.description` directly (the raw YouTube video description).
- `src/handlers/feed.rs:44` sets `<itunes:summary>` to the same raw description.
- `chrono::DateTime<Utc>` supports both `from_unix_timestamp(i64)` (for Unix epoch → DateTime) and `to_rfc2822()` (for RFC 2822 output). No additional crate needed.

## Goals / Non-Goals

**Goals:**
- Episodes in the feed are ordered by their precise YouTube upload timestamp, matching the original channel order exactly.
- Every `<pubDate>` in the RSS feed follows RFC 2822 format.
- Every episode description begins with the YouTube video link.
- Both `<description>` and `<itunes:summary>` are updated consistently.
- The feed remains reachable at the legacy `/{slug}/feed.xml` URL, serving the identical content as the canonical `/channels/{slug}/feed.xml`.
- New episodes get precise timestamps; existing episodes keep their current `published_at` (no DB migration).

**Non-Goals:**
- Backfilling precise timestamps for existing episodes. Only new episodes processed after this change will have second-precise dates.
- Changing the `published_at` column type or format in the database.
- Adding richer HTML formatting to descriptions.
- Changing the canonical feed URL linked by the frontend (`/channels/{slug}/feed.xml`).

## Decisions

### Decision 1: Use yt-dlp's `timestamp` field instead of `upload_date`

Add `timestamp: Option<i64>` to the `YtVideo` struct. In `process_episode`, use `Utc.timestamp_opt(ts, 0)` when `timestamp` is `Some`, falling back to the current `upload_date` → midnight UTC parsing when it's `None`.

**Why**: yt-dlp's `timestamp` is a Unix epoch with second precision, exactly what we need for correct ordering. The fallback ensures robustness if yt-dlp ever omits the field for some videos. `timestamp` is wrapped in `Option<i64>` with `#[serde(default)]` so the struct remains backward-compatible with yt-dlp outputs that lack the field.

**Alternative considered**: Trying to keep the current approach and add another ordering field. Rejected: `published_at` already serves as the date source for the feed; we should fix the root problem rather than work around it.

### Decision 2: Use `chrono::to_rfc2822()` instead of a manual format string

Replace `episode.published_at.to_string()` with `episode.published_at.to_rfc2822()` in `feed.rs`.

**Why**: `to_rfc2822()` is built into chrono, produces the exact format RSS 2.0 requires, and works correctly with any `DateTime<Utc>` regardless of whether it came from `upload_date` (midnight) or `timestamp` (precise).

### Decision 3: Prepend the YouTube URL as plain text, not HTML

The description becomes `{webpage_url}\n\n{description}` — a plain URL followed by a blank line, then the original description.

**Why**: plain URLs are recognized as tappable links by virtually all podcast clients, regardless of whether they render descriptions as plain text or HTML.

### Decision 4: Modify the description in-memory at feed generation time, not at storage time

The YouTube link is prepended only when building the RSS feed in `feed.rs`. Existing episodes keep their raw descriptions in the database unchanged.

**Why**: avoids a DB migration, avoids modifying historical data, and keeps the raw YouTube description available for other uses (frontend JSON API).

### Decision 5: Register the feed handler on both the canonical and the legacy route

In `web_feed` (`src/handlers/feed.rs`), register the existing `get_feed` handler on `/{slug}/feed.xml` in addition to `/channels/{slug}/feed.xml`, both wrapped in `BasicAuthGuard`. Both routes use the same `Info` extractor (a single `slug` path param) and the same handler, so they are guaranteed to produce identical feeds.

**Why**: older podcast subscriptions point at the pre-slug URL scheme; keeping the alias means those clients keep working without any reconfiguration. The `get_feed` handler resolves the channel by slug and is agnostic to the URL prefix, so reusing it is the minimal, zero-duplication change.

**Route conflict check**: `/{slug}/feed.xml` matches only two-segment paths ending in `feed.xml`. It cannot collide with `/api/...` (three+ segments), `/media/...` (registered in `config_services`), `/app/...` (SvelteKit SPA served by `af::Files`), or the `/` redirect. Actix-web routes are matched by registration order; the feed routes are registered inside `config_services`, and the SPA's `default_handler` in `main.rs` only serves unmatched paths under `/app`.

**Alternative considered**: Using `web::redirect` from the legacy URL to the canonical one. Rejected: a 3xx redirect on a feed URL is supported by fewer podcast clients than serving the feed body directly, and it would change the URL a client refreshes, defeating the purpose of the alias. Serving the same handler on both paths returns `200` with the feed XML from either URL.

### Decision 6: Return a plain `404 Not Found` for unknown feed slugs

`get_feed` returns `HttpResponse` directly on every path: `200` with the RSS body on success, `500 Internal Server Error` on a DB failure while reading episodes, and `404 Not Found` when the channel slug does not match any channel.

**Why**: the previous code propagated `Error`, but `Error::error_response` builds a JSON `CustomResponse` via `self.session.clone().unwrap()`, and the feed route never calls `set_session` (unlike the session-guarded API handlers in `channels.rs`). With `session == None` the `.unwrap()` panics, so an unknown slug produced a `500` instead of the intended `404`. Returning `HttpResponse` directly sidesteps the session-dependent error machinery, gives podcast clients a plain status code they understand, and makes both the canonical and legacy routes behave identically for an unknown slug.

## Risks / Trade-offs

- **[Risk] Timestamp precision change may reorder same-day episodes for existing feeds.** Podcast clients using `<guid>` (yt_id) will not re-download; they will simply show episodes in the corrected order after the next feed refresh.
- **[Risk] pubDate format change may cause clients to re-download episodes.** Some podcast clients use `<pubDate>` as part of the item identity. → Mitigation: the `<guid>` (yt_id) remains unchanged, so clients that respect GUID will not duplicate episodes.
- **[Trade-off] Description duplication.** The `webpage_url` is already stored as a column. Including it in the description duplicates the information but ensures it is visible where users expect it.

## Migration Plan

1. Add `timestamp` field to `YtVideo` struct in `src/models/ytdlp.rs`.
2. Replace `parse_date(&ytvideo.upload_date)` with timestamp-based conversion in `src/utils/worker.rs`; remove the `parse_date` function.
3. Fix `pubDate` format and prepend YouTube URL in `src/handlers/feed.rs`.
4. Register the legacy `/{slug}/feed.xml` route in `web_feed` (`src/handlers/feed.rs`).
5. `cargo build` in the container; redeploy.
6. Verify feeds (see `tasks.md`). Existing episodes keep their current dates; new episodes get precise timestamps.

**Rollback**: revert the changes in `ytdlp.rs`, `worker.rs`, and `feed.rs`. No DB migration to revert. Existing episodes retain their `published_at` unchanged.

## Open Questions

None.
