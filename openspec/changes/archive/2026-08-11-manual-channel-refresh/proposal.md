## Why

The backend refreshes channels only on the periodic worker cycle (every `sleep_time` hours). When a user adds a new channel or wants fresh episodes from an existing one, they must wait up to the full sleep period, and there is no way to trigger an immediate refresh from the UI.

## What Changes

- **BREAKING**: The worker's `process_channel` function becomes callable for a single channel from the API layer (it is currently private and only used inside the periodic loop).
- Add a backend endpoint `POST /api/1.0/channels/{slug}/update/` that resolves a channel by slug and triggers its episode refresh asynchronously (fire-and-forget task), so the request returns immediately without waiting for downloads to finish.
- Trigger an immediate refresh automatically right after a new channel is created (`POST /api/1.0/channels/`), so the new channel starts downloading episodes without waiting for the periodic cycle.
- Add a "Refresh" button to the episodes page (`EpisodesView.vue`) that calls the new endpoint for the channel being viewed and shows a loading/notification state.

## Capabilities

### New Capabilities
- `manual-channel-refresh`: On-demand channel update — a backend endpoint that refreshes a single channel's episodes out of cycle, an automatic refresh fired when a channel is created, and a frontend "Refresh" button on the episodes page wired to the endpoint.

### Modified Capabilities
- `vue3-spa`: The episodes screen gains a refresh control that triggers a channel update request; the channel creation flow signals the backend to start updating the new channel immediately.

## Impact

- **Backend**: `src/utils/worker.rs` — expose a public single-channel refresh function (reuse the existing `process_channel`/`clean_channel` logic); `src/handlers/channels.rs` — add the update endpoint and call the refresh on create; `src/handlers/mod.rs` — register the new route.
- **Frontend**: `frontend/src/lib/api/client.ts` — add `refreshChannel(slug)`; `frontend/src/views/EpisodesView.vue` — add the Refresh button wired to the API with loading/notification feedback.
- No design-system changes; the button reuses existing `AppButton` and notification/loading stores.
