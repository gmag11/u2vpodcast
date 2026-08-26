## 1. Store modes

- [x] 1.1 In `frontend/src/stores/player.ts` add mode state: `shuffle: boolean` and `repeat: 'none' | 'all' | 'one'` (default none).
- [x] 1.2 Keep the authored order as `sourceOrder` (the seeded `upNext`); introduce an injectable `random` (default `Math.random`) used by a Fisher–Yates helper for the shuffled consumption order.
- [x] 1.3 `toggleShuffle()`: on enable build a shuffled order copy; on disable restore the authored order; never mutate `upNext` itself.
- [x] 1.4 `cycleRepeat()`: none → all → one → none, returning the new mode.
- [x] 1.5 In `onEnded`/`advance()`: repeat-one replays the finished episode from start (seek 0, `fromStart` semantics per step 3); repeat-all rebuilds the consumption order from the seeded source (re-shuffled when shuffle is active) when exhausted; repeat-none keeps current stop behavior.
- [x] 1.6 Persist `shuffle` + `repeat` in the existing `u2vpodcast.up-next.v1` localStorage payload and restore them on load.

## 2. Bar controls

- [x] 2.1 In `frontend/src/components/PersistentPlayer.vue` add a shuffle button (`PhShuffle` icon) toggling `toggleShuffle()`, visually active when on.
- [x] 2.2 Add a repeat button cycling `cycleRepeat()` with distinct icons for none (off), all (`PhRepeat`), and one (`PhRepeatOnce`), visually indicating the active mode.
- [x] 2.3 Add en/es tooltips for shuffle and repeat modes in `frontend/src/i18n/locales/` and keep locale parity.

## 3. Tests

- [x] 3.1 In `frontend/src/stores/player.test.ts` inject a seeded random and verify: shuffle permutes the order without losing/duplicating items; disabling shuffle restores authorship; repeat-one replays the same episode; repeat-all rebuilds the queue after the last item (and re-shuffles per cycle); repeat-none stops and clears.
- [x] 3.2 Verify mode persistence round-trips through `queue.storage.ts`.

## 4. Verification

- [x] 4.1 `pnpm test` and `pnpm build` in `frontend/`.
- [x] 4.2 Manual: enable shuffle on a playlist and confirm a random-but-complete pass; confirm repeat-all loops the queue; confirm repeat-one replays the current episode; confirm modes survive a reload.