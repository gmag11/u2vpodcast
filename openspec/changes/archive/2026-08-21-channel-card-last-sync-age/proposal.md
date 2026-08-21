## Why

Channel cards already show a sync-status dot (top-left) and the age of the last episode (top-right), but there is no indication of how long ago the channel was last refreshed. Users cannot tell at a glance whether a channel's local data is recent or stale. Showing the hours since the last sync makes staleness immediately visible.

## What Changes

- Add a small, non-interactive badge to the **bottom-left corner** of each channel card showing the elapsed time since the channel's last sync (derived from `last_sync_at`).
- Format the elapsed time with truncated units, beginning in hours (e.g. `1h`, `5h`), escalating to days, weeks, months and years for older syncs.
- Channels that have never been synced (null `last_sync_at`) show no badge, consistent with existing badge behavior.
- Introduce a reusable helper `lastSyncAge()` mirroring the existing `lastEpisodeAge()` util, with unit tests.
- No backend or API change: `last_sync_at` is already exposed by the channel API.

## Capabilities

### New Capabilities
- `channel-card-sync-age`: Defines the "last sync age" badge on channel cards, showing truncated hours (and larger units) since the channel's last sync, or no badge for never-synced channels.

### Modified Capabilities
<!-- None: this adds a new, distinct UI element. The existing `channel-sync-status` (dot) and `channel-card-age-badge` (last-episode age) capabilities are unchanged. -->

## Impact

- **Frontend**: `frontend/src/components/ChannelCard.vue` (add badge markup in bottom-left), new `frontend/src/lib/utils/channel.sync.age.ts` helper, `frontend/src/types.ts` unchanged (`last_sync_at` already present).
- **Tests**: new unit tests for the helper (mirroring `channel.age.test.ts`).
- **No backend change**: `last_sync_at` already in `Channel` API payload.
