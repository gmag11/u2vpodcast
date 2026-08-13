## Why

The channel cover image URL is captured once at channel creation time from the YouTube `og:image` metadata. When a channel is renamed or rebranded on YouTube, the stored cover stays stale, and there is no way to re-read it without triggering a full episode refresh. Users need a lightweight way to re-fetch the cover image URL on demand.

## What Changes

- Add a lightweight authenticated backend endpoint to re-read the YouTube cover image URL for a single channel and persist it to the `channels.image` column, without touching episodes.
- Add a small cover-refresh button on each `ChannelCard` that calls the new endpoint, shows a loading state while the request is in flight, and surfaces a success/error notification.
- The button emits an event so the owning view (ChannelsView) performs the API call and updates the channel list with the returned channel.

## Capabilities

### New Capabilities

- `channel-cover-refresh`: Ability to re-fetch and persist a channel's YouTube cover image URL on demand, exposed as an authenticated endpoint and a small button on the channel card.

### Modified Capabilities

<!-- No existing spec-level requirement changes; this is a new capability. -->

## Impact

- Backend: new handler in `src/handlers/channels.rs`, new model method in `src/models/channel.rs` to update the image column, route registration in `src/handlers/mod.rs`. Reuses `YTInfo::new` / `get_image` from `src/models/ytinfo.rs`.
- Frontend: new API method in `frontend/src/lib/api/client.ts`, button in `frontend/src/components/ChannelCard.vue`, wiring in `frontend/src/views/ChannelsView.vue`, notification store reuse.
- No DB schema change (existing `image` column).
