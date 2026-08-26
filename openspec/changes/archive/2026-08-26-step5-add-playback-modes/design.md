## Context

After step 4, the up-next queue is seeded from a playlist (or a list view) in display order. The player store (`frontend/src/stores/player.ts`) consumes it linearly in `onEnded → advance()`, and the persistent bar (`PersistentPlayer.vue`) shows prev/next + the queue panel. This change adds playback modes on top of that queue; everything stays client-side.

## Goals / Non-Goals

**Goals:**
- Shuffle on/off over the up-next queue.
- Repeat none/all/one handled when the last episode ends.
- Mode toggles in the persistent bar reflecting store state.
- Deterministic, testable logic (seeded shuffle in tests).

**Non-Goals:**
- Server-side modes or cross-device mode sync.
- Smart shuffle (no-repeat-history weighting), crossfade, gapless audio.
- Changing the saved playlist order itself when shuffling (playlist stays as authored).

## Decisions

### Decision 1: Store state

```
modes = reactive({ shuffle: false, repeat: 'none' | 'all' | 'one' })
sourceOrder: number[]       // indices of the seeded queue as authored
playOrder: number[]         // indices actually consumed (shuffled copy when active)
cursor in playOrder         // current position within the order
```

The store keeps the *base* queue (`upNext` = authored order from seed) and a consumption order. Enabling shuffle generates `playOrder` from `sourceOrder` via Fisher–Yates; disabling restores `sourceOrder`. Shuffle never mutates the seeded `upNext` list.

**Why**: separating authored order from consumed order is the standard model (playlist UI stays stable while playback previews shuffle). Deterministic Fisher–Yates with an injectable PRNG makes tests reproducible.

### Decision 2: Repeat semantics in `onEnded`

- `repeat: 'one'` → replay the current episode (seek 0, no queue consumption).
- `repeat: 'all'` → when consumed order is exhausted, rebuild `playOrder` from `sourceOrder` (re-shuffled if shuffle is active) and continue.
- `repeat: 'none'` (default) → current step-2 behavior: advance, then stop/clear on exhaustion.
- Shuffle + repeat-all interplay: each cycle re-shuffles, so consecutive cycles differ.

**Why**: matches podcast-client expectations; re-shuffle-per-cycle is the simplest correct behavior and avoids the classic "listen forever to the same first track" bug.

### Decision 3: UI toggles in the persistent bar

- Repeat button cycles `none → all → one → none` on click (icons `PhRepeat` / `PhRepeatOnce`; active state tinted like the volume/speed controls).
- Shuffle button toggles on/off (`PhShuffle`), active state when on.
- Modes persisted in `localStorage` under the same `u2vpodcast.up-next.v1` payload (extended), so reload keeps the mode.

**Why**: buttons are already the bar's interaction language; persisting modes costs nothing on the existing storage path.

## Risks / Trade-offs

- **[Risk] Shuffle determinism in tests.** Use an injectable `random` (default `Math.random`) in the store's shuffle so tests seed it.
- **[Trade-off] No "history-aware" shuffle.** Acceptable; Fisher–Yates covers the stated need.
- **[Risk] Repeat-one never advances the queue; position must reset.** Explicit `seek(0)` + clear resume flag is part of the task.

## Migration Plan

1. Extend the store: modes state, shuffle copy, repeat handling in `onEnded`, persistence.
2. Add toggle buttons in `PersistentPlayer.vue` + i18n tooltips (en/es).
3. Unit tests: seeded shuffle permutation, repeat-all cycle rebuild, repeat-one replay, empty-queue edges.
4. Manual verification per `tasks.md`.

**Rollback**: revert frontend changes; no backend/DB impact.

## Open Questions

None.