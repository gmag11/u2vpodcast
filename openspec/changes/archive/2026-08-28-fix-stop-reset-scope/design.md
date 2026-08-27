# Fix: Scope Stop Progress Reset to the Card Control — Design

## Context

The player store (`frontend/src/stores/player.ts`) exposes a single stop control. Callers differ: the persistent player bar always calls `stop()` with no target (acts on the current episode), while the episode cards call `stop(episode)` to act on the card's own episode. An earlier change removed the position reset from both controls; the user clarified that the reset belongs to the card control only when the episode is not reproducing, while the persistent bar's stop is a pure stop that never touches saved positions.

## Goals / Non-Goals

**Goals:**
- `stop(target)` (card): halt a reproducing current episode (keep position); otherwise reset the target episode's saved position to 0 keeping the listened mark.
- `stop()` (persistent bar): halt when reproducing; converge to stopped when not; never reset.
- Restore `resetPosition` (deleted by the previous iteration) with its user-gesture semantics.
- Re-enable the card's stop button on non-current episodes.
- Update `playback-progress` and `episode-cards` specs to this corrected semantics.

**Non-Goals:**
- No change to "start over" (`fromStart`), the only in-player way to clear a position.
- No change to internal stops (end of queue, session teardown).
- No change to `unmark` (explicit listened-clearing flow).
- No backend changes.

## Decisions

### D1: `stop` switches on the caller's target

```ts
function stop(target?: Episode) {
    const targetEpisode = target ?? currentEpisode.value;
    if (!targetEpisode) return;
    const isCurrentTarget = target == null || targetEpisode.id === currentEpisode.value?.id;
    const el = audio;
    const reproducing = isCurrentTarget && !stopped.value && el != null && !el.paused;
    if (reproducing) {
        haltPlayback();                     // stop while playing → halt, keep position
        return;
    }
    if (target != null) {
        resetPosition(targetEpisode);       // card stop on a non-reproducing episode → reset
        return;
    }
    // Persistent-bar stop: pure stop, never resets a saved position.
    stopped.value = true;
    playing.value = false;
    currentTime.value = 0;
    if (el) { el.pause(); el.currentTime = 0; }
}
```

`resetPosition` is restored unchanged (writes 0 via `api.updateEpisodeProgress`, keeps the listened mark, no-ops when already 0).

### D2: Card stop button enabled for non-current episodes

Both stop buttons in `EpisodeCard.vue` go back to `:disabled="isCurrent && player.loading"`: with the reset restored, a non-current card's stop is the "rewind this episode" affordance and must be clickable.

### D3: Spec deltas

- `playback-progress`: the stop requirement now splits persistent-bar stop (never resets) from card stop (resets a non-reproducing episode).
- `episode-cards`: the shared-player requirement reflects the same split, including the persistent bar's never-reset guarantee.

## Risks / Trade-offs

- A card stop on the current, stopped episode resets it (as before this exercise) — that is the requested card semantic; users who want a pure halt use the persistent bar's stop.
- `resetPosition` writes through the API even when offline (the write fails silently via `.catch`), leaving the in-memory position 0 — consistent with the pre-existing behavior.