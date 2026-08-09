## Why

The RSS feed at `/channels/{channel_id}/feed.xml` is generated with `Episode::read_all(&data.pool)`, which returns every episode in the database regardless of channel. As a result each feed mixes in episodes from all channels: a subscriber of one channel sees chapters of every other channel, and the `<enclosure>` URLs are wrong too (they point at the feed's `channel_id` with a `yt_id` that belongs to another channel), so podcast clients fail to download most episodes. Each feed must contain exclusively the episodes of its own channel.

## What Changes

- **Filter feed episodes by channel.** In `src/handlers/feed.rs`, replace the `Episode::read_all(&data.pool)` call in `get_feed` with `Episode::read_episodes_for_channel(&data.pool, channel_id)` (already implemented in `src/models/episode.rs:103`), so every item in a channel's feed is an episode whose `channel_id` equals the requested channel.
- **Correct `<enclosure>` URLs.** Because episodes are now filtered by channel, each `<enclosure>` URL `{url}/media/{channel_id}/{yt_id}.mp3` points at a real file for that channel (the media files are stored under `/media/{channel_id}/`). No change to the URL construction is needed — filtering is what fixes the mismatches.
- **Ordering.** `read_episodes_for_channel` orders by `published_at DESC`, which is the desired newest-first feed order (replacing the current unsorted `read_all`).
- No public API, config, dependency, or DB changes.

## Capabilities

### New Capabilities
- `rss-feeds`: generation of each channel's RSS feed, including that a feed contains exclusively its own channel's episodes, in newest-first order, with correct per-channel enclosure URLs.

### Modified Capabilities
<!-- None: the existing `route-protection` capability only covers access control and is unaffected by this content fix. -->

## Impact

- **Code**: `src/handlers/feed.rs` (`get_feed` uses `Episode::read_episodes_for_channel` instead of `Episode::read_all`).
- **APIs**: `/channels/{channel_id}/feed.xml` now serves only that channel's episodes. Existing subscriptions need no reconfiguration (URLs unchanged); feed contents change to the correct per-channel episodes.
- **Dependencies**: none.
- **DB**: none (no migration — the query already exists).
- **Frontend**: none (the SPA only links to the feed URL).
