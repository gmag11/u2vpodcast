## Why

The current player only offers coarse playback speed presets (0.5x, 1x, 1.25x, 1.5x, 2x) and never remembers the choice: after a reload the player always comes back at 1x, and switching between channels resets nothing because there is no per-channel memory. Listeners who regularly speed up certain channels (or need fine-grained values like 1.35x or 1.7x) have no way to set or keep that preference.

## What Changes

- The player speed selector becomes a fine-grained control: a stepper with **+** and **−** buttons that adjusts the rate in half-tenths (0.05 steps), in addition to the existing presets. Values such as 1.35x or 1.7x become reachable.
- Playback speed becomes a **per-channel preference** stored server-side (`channels.playback_speed`), so the same value applies to every episode of a channel and survives reloads and other devices.
- Starting playback of an episode applies the saved speed of its channel; changing the speed while playing overwrites the saved value for that channel immediately.
- Switching from an episode of one channel to an episode of a different channel always loads and applies the new channel's saved speed — both when the previous episode ends (auto-advance) and when the user skips manually — so the previous channel's playback rate is never carried over.
- Backend: new migration column on `channels`, channel payload gains the field, episode payloads expose the channel's saved speed, and a new endpoint to update it.
- Frontend: `player` store seeds and tracks speed per channel slug, applies the channel speed on every episode load (fresh play, resume, end-of-episode auto-advance, manual skip, restored queue), and persists changes fire-and-forget via the API (mirroring playback-progress).

## Capabilities

### New Capabilities
- `per-channel-playback-speed`: Server-side storage of a playback speed per channel, delivery of that speed in channel and episode payloads, an update endpoint with validation, and the semantics that every episode of a channel starts at the saved speed and that manual changes overwrite the saved value.

### Modified Capabilities
- `persistent-audio-player`: The persistent bar's playback speed control changes from a fixed-preset dropdown to a stepper panel with +/− controls (0.05 steps) beside the presets; the shared player's `speed` state is (re)applied per channel whenever an episode starts.

## Impact

- **Database**: new `playback_speed` column on `channels` (migration; default 1.0).
- **Backend API** (`src/`): `Channel` model + `from_row`, episode payload builders (JOIN + channel-episodes handler), new `PUT /api/1.0/channels/{slug}/playback_speed/` endpoint with bounds validation, route registration in `handlers/mod.rs`.
- **Frontend** (`frontend/src/`): `stores/player.ts` (speed state per channel, apply-on-play, save-on-change, queue/restore handling), `components/PersistentPlayer.vue` (stepper UI), `lib/api/client.ts` (new API method), `types.ts` (`playback_speed` field), plus i18n labels and tests.
- **Tests**: backend handler/model tests for the new endpoint and payload; frontend store tests for apply-on-play, save-on-change, and stepping bounds; component test updates for the new speed panel.