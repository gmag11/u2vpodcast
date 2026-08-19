## Why

In the history list, the channel name on each episode card is plain text. Users who want to browse a channel's episodes from the history screen cannot click through; they must go back to the channels list first.

## What Changes

- Turn the channel name shown on compact episode cards (the history list variant) into a link to that channel's episode list.
- The link navigates to the existing episodes route (`/app/:channelId`) using the episode's `channel_id`.

## Capabilities

### New Capabilities
- `channel-links-from-history`: making the channel name on history episode cards navigable to the channel's episode list.

### Modified Capabilities
<!-- No existing spec requirement covers the channel label, so no delta spec is needed -->

## Impact

- `frontend/src/components/EpisodeCard.vue`: the compact channel-title `<p>` becomes a `<RouterLink>` to `{ name: 'episodes', params: { channelId: String(episode.channel_id) } }`, mirroring `ChannelCard.vue`.
- No router, API, store, or backend changes; the episodes route and `Episode.channel_id` already exist.
