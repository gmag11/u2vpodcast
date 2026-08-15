## Why

Today RSS feeds are per-channel only: to follow new episodes a subscriber must subscribe to each channel's feed separately, one by one. There is no single feed that aggregates every episode from every channel in chronological order. The history screen already shows the cross-channel view in the SPA, so it is the natural place to expose an aggregated feed that mirrors that same view in any podcast client.

## What Changes

- Add a protected backend endpoint `GET /feed.xml` that returns a single RSS document containing every episode from every channel, ordered by `published_at` descending (newest first), reusing the existing all-episodes query.
- Each feed item's `<enclosure>` points at the owning channel's media file (`{url}/media/{slug}/{yt_id}.mp3`) and the item title is prefixed with the channel title so episodes are distinguishable in the aggregated feed.
- Add a download link with the RSS icon on the history screen (`HistoryView.vue`) pointing at the global feed, matching the existing per-channel feed link in `ChannelCard.vue`.

## Capabilities

### New Capabilities
- `global-feed`: a protected RSS endpoint that aggregates every episode from every channel in publication order, plus a download link with the RSS icon on the history screen.

### Modified Capabilities
- None.

## Impact

- **Code**:
  - Backend (`src/handlers/feed.rs`): a new `get_global_feed` handler and its route registration in `web_feed`. No model changes: `Episode::read_all_with_channels` already returns every episode across channels with `channel_slug` and `channel_title`.
  - Frontend (`frontend/src/views/HistoryView.vue`): an RSS icon link in the header area pointing at the global feed URL.
- **APIs**: new `GET /feed.xml` (protected by the existing `SessionOrBasicAuth` middleware, same as per-channel feeds); no changes to existing endpoints.
- **Dependencies**: none (reuses the `rss` crate and `baseEndpoint`).
- **DB**: none (read-only over existing `episodes` + `channels` tables).
- **Frontend**: updated history screen with the feed link.
