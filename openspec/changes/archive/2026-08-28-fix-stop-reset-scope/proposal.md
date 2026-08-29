# Fix: Scope Stop Progress Reset to the Card Control

## Why

The persistent player bar's stop button reset an episode's saved position to 0, destroying the resume point when the user merely wanted to halt. The reset-to-zero semantic belongs exclusively to the episode card's stop control, which is the "rewind this episode" affordance (the card's stop clears a non-reproducing episode's position). The initial fix removed the reset from both controls; this change restores it on the card while keeping the player bar's stop a pure stop.

## What Changes

- `player.stop(target?)` distinguishes the two controls:
  - **Card stop (with target)**: halts a reproducing current episode keeping its position; otherwise (non-reproducing — a non-current card, or the current episode stopped/paused) resets that episode's saved position to 0 keeping the listened mark.
  - **Player bar stop (no target)**: purely halts — halt when reproducing, converge to stopped state otherwise — never touching any saved position.
- The card's stop button is enabled again for non-current episodes (it is the reset affordance).

## Capabilities

### New Capabilities
<!-- None -->

### Modified Capabilities
- `playback-progress`: The stop requirement is corrected so the player bar's stop never resets a saved position while the card's stop resets the position of a non-reproducing episode.
- `episode-cards`: The "Play/pause and stop bound to the shared player" requirement is corrected: the card's stop resets a non-reproducing episode's position; the persistent bar's stop never does.

## Impact

- **Frontend player store** (`frontend/src/stores/player.ts`): restore `resetPosition`, rewrite `stop` with the target-based reset scoping.
- **Component** (`frontend/src/components/EpisodeCard.vue`): re-enable stop on non-current cards.
- **Tests**: `player.test.ts` (player-bar keeps vs card resets), `EpisodeCard.test.ts` (stop enabled on non-current).
- No backend changes.