## Why

When an episode finishes in the web player the shared store calls `stop()`, so listening ends there and the user must manually start the next episode. For podcast-style listening the player should flow from one episode to the next without intervention.

## What Changes

- The player store learns an implicit playback queue: the ordered list behind the starting episode (the channel's episode list or the global feed list that is visible when play was pressed).
- On `ended` the store plays the next episode from that queue; when the queue is empty it stops as today.
- "Next" is defined by the list the user pressed play from (channel page order or global-feed order, honoring whatever search/filter is currently applied), not by any hidden global rule.
- The current implementation of `onEnded() -> stop()` in `frontend/src/stores/player.ts` is replaced by `onEnded() -> advance or stop`.
- `EpisodeCard` gains an optional `list` prop so the views can hand the visible (filtered) list to the store when starting playback.
- No backend change, no migration.

## Capabilities

### New Capabilities
- `auto-advance`: when the shared audio element reaches the end, the player advances to the next episode in the context list that started playback, or stops when no next episode exists.

### Modified Capabilities
- None.

## Impact

- **Code**: `frontend/src/stores/player.ts` (queue state, `play(episode, list?)`, `advance()`/`playNext()` wired to the existing `ended` listener), `frontend/src/components/EpisodeCard.vue` (optional `list` prop, pass current/filtered list on play), `frontend/src/views/EpisodesView.vue` and `frontend/src/views/HistoryView.vue` (pass `filteredEpisodes` to cards).
- **APIs**: none.
- **Dependencies**: none.
- **DB**: none.
- **Frontend**: unit tests for the queue/advance logic in `frontend/src/stores/player.test.ts`; manual sync verification of card/progress states unchanged.