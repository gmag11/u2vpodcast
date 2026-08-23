## Why

Step 1 (`auto-advance`) makes the player flow to the next episode, but the queue is invisible and implicit: the user cannot see what is coming next, cannot remove a wrong episode, and loses the queue on a page reload. A podcast-style web player needs a visible, editable "up next" queue to stay.

## What Changes

- The player store exposes the queue as public state (`upNext: Episode[]`) with manipulation API: `skipNext`, `playPrevious`, `removeFromQueue`, `clearQueue`, and `queue` seeding.
- The persistent bottom player bar gains an "Up next" panel (collapsible drawer or popover) listing upcoming episodes with remove action, plus next/previous controls in the bar itself.
- When playback starts from a list, the visible list seeds the queue (kept in current display order, excluding the started episode); starting a single episode without a list leaves the queue as-is.
- The queue is persisted to `localStorage` (serialized full episode objects) and rehydrated on app load, so a reload does not clear the queue.
- `onEnded` now consumes the queue and persists the updated queue; on reaching the last item the player stops and the queue is emptied.
- No backend change, no migration.

## Capabilities

### New Capabilities
- `up-next-queue`: a visible, editable, persisted playback queue feeding the auto-advance behavior; previous/next navigation and removal surface in the persistent player bar.

### Modified Capabilities
- `persistent-audio-player`: the persistent bottom bar additionally exposes next/previous controls and a queue panel showing what is coming up.

## Impact

- **Code**: `frontend/src/stores/player.ts` (public `upNext` state, queue mutations, localStorage persistence via a small serialize/rehydrate helper), `frontend/src/components/PersistentPlayer.vue` (next/prev buttons, collapsible "Up next" panel with remove), `frontend/src/components/EpisodeCard.vue` + both list views (seed the queue with the visible list on play).
- **APIs**: none.
- **Dependencies**: none (plain `localStorage` + existing Vue/Pinia primitives).
- **DB**: none.
- **Frontend**: unit tests for queue mutations and localStorage round-trip in `frontend/src/stores/player.test.ts`; `PersistentPlayer.vue` interaction tests.