## Why

With saved playlists (step 4) the queue is long and linear. Podcast clients offer playback modes — shuffle and repeat (all/one) — that make long queues practical. Without them the listener must manually reorder or restart playback.

## What Changes

- The player store gains playback mode state: `shuffle` (off/on) and `repeat` (none/all/one), stored on the queue.
- Shuffle builds a randomized copy of the up-next queue (Fisher–Yates) while the playlist order itself is untouched; the "next" pointer walks the shuffled order.
- Repeat-all re-queues the playlist (in its current mode order) when the last episode finishes; repeat-one replays the finished episode.
- Toggle controls appear in the persistent player bar (repeat cycles none→all→one) and unit tests cover the mode logic.
- No backend change, no migration; modes live entirely in the frontend player store.

## Capabilities

### New Capabilities
- `playback-modes`: shuffle, repeat-all, and repeat-one applied to the up-next queue.

### Modified Capabilities
- `persistent-audio-player`: the persistent bar exposes shuffle and repeat toggles reflecting the store state.

## Impact

- **Code**: `frontend/src/stores/player.ts` (mode state, shuffle copy, repeat handling in `onEnded`), `frontend/src/components/PersistentPlayer.vue` (toggle buttons + active styles), `frontend/src/i18n/` (en/es tooltips).
- **APIs**: none.
- **Dependencies**: none.
- **DB**: none.
- **Frontend**: unit tests for shuffle determinism, repeat-all re-queue, repeat-one replay, and empty-queue edge cases.