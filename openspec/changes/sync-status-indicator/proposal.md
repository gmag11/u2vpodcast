## Why

Channel cards give no signal about whether the last sync succeeded or failed. A channel can silently stop updating (403/429/503, bad cookies, etc.) with no visible clue. Users need a quick way to spot channels whose last sync failed.

## What Changes

- Record the outcome of each channel sync in the database: a timestamp of the last sync attempt and whether it succeeded or failed (optionally with the error message).
- Persist sync status for both sync paths: the background worker and the manual per-channel refresh endpoint.
- Expose the sync status fields in the channel API payload so the SPA can render them.
- Render a small indicator dot in the top-left corner of each channel card: green when the last sync succeeded, red when it failed.

## Capabilities

### New Capabilities
- `channel-sync-status`: Tracks the outcome (success/failure, timestamp) of the last sync per channel and surfaces it on the channel cards as a green/red indicator dot.

### Modified Capabilities
<!-- None: the channel list payload and card rendering are covered by the new capability. -->

## Impact

- Backend: `channels` table (new columns via migration), `Channel` model (`src/models/channel.rs`), sync/worker flow (`src/utils/worker.rs`), channel handlers (`src/handlers/channels.rs`).
- Frontend: `Channel` type (`frontend/src/types.ts`), `ChannelCard.vue` (top-left indicator), `ChannelsView.vue` (refresh flow that will need to propagate the new status).
- No breaking changes: new nullable fields and a non-blocking visual indicator only.
