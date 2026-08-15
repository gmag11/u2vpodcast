## Why

Episode metadata and feed URLs have four issues that degrade the podcast experience:

1. **Wrong feed episode order.** All episodes published on the same YouTube day end up with the same `published_at` (midnight UTC) because `parse_date` only uses `upload_date` (`YYYYMMDD`), discarding the time component. When SQLite orders by `published_at DESC`, episodes from the same day appear in arbitrary order — not matching the true sequence on YouTube.

2. **`pubDate` uses an invalid format.** The feed emits `episode.published_at.to_string()`, which produces `"2024-03-15 00:00:00 UTC"`. Podcast clients expect RFC 2822 format (e.g., `"Fri, 15 Mar 2024 14:30:00 +0000"`). The non-standard format causes some clients (notably Apple Podcasts) to ignore the field and use the feed fetch time instead, making it appear as if the server sync time is used.

3. **No YouTube link in episode descriptions.** The RSS `<description>` and `<itunes:summary>` contain the raw YouTube video description, but there is no link back to the original video on YouTube. Listeners who want to jump from the podcast to watch the video must manually search for it.

4. **Legacy feed URLs stop working.** Older versions of the application served each channel's feed at `/{slug}/feed.xml`, while the current version serves it at `/channels/{slug}/feed.xml`. Podcast clients subscribed to the old URL (e.g., `https://<host>/<slug>/feed.xml`) no longer receive updates after the URL scheme changed. The feed must remain reachable at both URLs for backwards compatibility; the frontend keeps linking the canonical `/channels/{slug}/feed.xml`.

## What Changes

- **Use `timestamp` instead of `upload_date` for precise episode ordering.** Add `timestamp: Option<i64>` to the `YtVideo` struct in `src/models/ytdlp.rs`, and in `src/utils/worker.rs` replace `parse_date(&ytvideo.upload_date)` with a conversion from the Unix epoch `timestamp` (falling back to `upload_date` if absent). This gives each episode a second-precise `published_at`, so feed ordering matches YouTube exactly.

- **Fix `pubDate` format.** In `src/handlers/feed.rs`, replace `episode.published_at.to_string()` with `episode.published_at.to_rfc2822()` so the field follows RFC 2822 (the RSS 2.0 standard for `<pubDate>`).

- **Prepend YouTube video link to episode description.** In `src/handlers/feed.rs`, prepend the YouTube video URL (`episode.webpage_url`) at the top of the episode description inside both `<description>` and `<itunes:summary>`, so users can jump from the podcast app to the original video on YouTube. The link is formatted as a plain URL followed by a blank line, keeping compatibility with podcast clients that render descriptions as plain text or HTML.

- **Serve the feed at the legacy URL too.** Register the existing `get_feed` handler on a second route `/{slug}/feed.xml` (in addition to `/channels/{slug}/feed.xml`), both protected by `BasicAuthGuard`, so podcast clients subscribed to the old URL continue to receive the same feed. The frontend continues to link the canonical `/channels/{slug}/feed.xml`; the alias adds no new API surface.

## Capabilities

### Modified Capabilities
- `rss-feeds`: episode `<item>` entries are now ordered by their precise YouTube timestamps, use RFC 2822 `pubDate`, include a YouTube video link at the top of the description, and the channel feed is served both at `/channels/{slug}/feed.xml` and at the legacy `/{slug}/feed.xml`.

## Impact

- **Code**: `src/models/ytdlp.rs` (add `timestamp` field to `YtVideo`), `src/utils/worker.rs` (parse timestamp in `process_episode`, remove `parse_date`), `src/handlers/feed.rs` (date format, description construction, and a second feed route).
- **APIs**: `/channels/{slug}/feed.xml` — episode ordering changes for same-day episodes, `pubDate` format changes to RFC 2822, and `<description>` + `<itunes:summary>` now start with the YouTube video URL. New alias route `/{slug}/feed.xml` serves the identical feed for backwards compatibility.
- **Dependencies**: none. `chrono::DateTime<Utc>` already supports `from_unix_timestamp` conversion and `to_rfc2822()`.
- **DB**: none (existing episodes keep their current `published_at`; only new episodes get precise timestamps).
- **Frontend**: none (the SPA already links the canonical `/channels/{slug}/feed.xml`).
