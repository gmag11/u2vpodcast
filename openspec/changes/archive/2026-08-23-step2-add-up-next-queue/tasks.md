## 1. Queue store API

- [x] 1.1 In `frontend/src/stores/player.ts` make `upNext` public state and add a private `playStack: Episode[]` for the previous control.
- [x] 1.2 Add mutations: `removeFromQueue(episodeId)`, `clearQueue()`, `skipNext(markCurrent?: boolean)` (shift + play; when `markCurrent` is true, mark the finished episode listened via the step-3 save path), `playPrevious()` (dual: when `currentTime > 3s` restart the current episode via `seek(0)`; otherwise pop `playStack` and play without re-seeding), and seed on play: when `list` provided replace `upNext`; when omitted keep it.
- [x] 1.3 Update `advance()` from step 1: shift, push the finished episode onto `playStack`, play, persist; on empty queue stop and clear.
- [x] 1.4 Keep `playStack` bounded (e.g. keep last 50) to avoid unbounded growth in long sessions.

## 2. Persistence helper

- [x] 2.1 Create `frontend/src/lib/utils/queue.storage.ts` with `saveQueue(payload)` and `loadQueue()` using `localStorage` key `u2vpodcast.up-next.v1`; wrap serialization/parsing in try/catch and return `null`/defaults on failure.
- [x] 2.2 Combine `upNext` + `playStack` in the stored payload and rehydrate both in the store setup; persist after every mutation.
- [x] 2.3 Add tests for the storage round-trip and malformed-payload handling.

## 3. Persistent player UI

- [x] 3.1 In `frontend/src/components/PersistentPlayer.vue` add a previous button (the next button already exists beside the stop control from step 1 and stays wired to skip/advance); disable next when `upNext` is empty; keep previous enabled while an episode is loaded (it can always restart the current episode) and only disable it when nothing is loaded; wire to `playPrevious`/`skipNext`.
- [x] 3.2 Implement next long-press on the button: `pointerdown` starts a 500ms timer; `pointerup` before it fires `skipNext()` (short); crossing it fires `skipNext({ markCurrent: true })` (long) and suppresses the release action. Ensure the timer is cleaned up on `pointerleave`/unmount; enter/space keeps the short action for keyboard users.
- [x] 3.3 Add a queue toggle button (e.g. `PhList` icon) opening an "Up next" popover (radix-vue `DropdownMenuRoot` or similar) listing `upNext` with thumbnail, title, channel, count, per-item remove (→ `removeFromQueue`) and a "clear all" (→ `clearQueue`).
- [x] 3.4 Show an empty-state line in the popover when the queue is empty (i18n string in en/es).
- [x] 3.5 Update the auto-hide watch in `PersistentPlayer.vue` to include `upNext.length`: when stopped with a non-empty queue the bar stays visible; the 10s hide timer arms only once the queue empties.
- [x] 3.6 Persist and restore the current episode alongside the queue; show the bar in queue-only mode when a queue is restored without a current episode (neutral title, play disabled) so the queue stays reachable.
- [x] 3.7 Bind the bar to the session in `App.vue`: render `<PersistentPlayer v-if="auth.isAuthenticated" />` and stop playback when the session disappears (logout), so the player is never available on the login screen.

## 4. Seed wiring (reuse step 1)

- [x] 4.1 Confirm `EpisodesView.vue` and `HistoryView.vue` still pass `:list="filteredEpisodes"`; adjust only if behavior changed around orphan-play queue keeping.

## 5. Tests

- [x] 5.1 Extend `frontend/src/stores/player.test.ts`: seed replaces queue; play without list keeps queue; `removeFromQueue`/`clearQueue`; `skipNext` (plain skips without marking, with `markCurrent: true` marks listened); `playPrevious` dual behavior (currentTime > 3s → restart at 0, ≤ 3s → pop `playStack`); advance pushes to `playStack`; stop clears and persists empty.
- [x] 5.2 Test that `saveQueue`/`loadQueue` tolerate corrupt data and that rehydration restores a previously saved queue.
- [x] 5.3 Component test for the bar (per `AppHeader.test.ts` pattern with `@vue/test-utils`): next/prev disabled states, popover rendering, and long-press via fake timers — release before 500ms fires short skip, hold past 500ms fires skip + listened mark.
- [x] 5.4 Component test for the stay-visible rule: stopped with a non-empty queue keeps the bar rendered; clearing the queue arms the hide timer (fake timers) and the bar disappears.
- [x] 5.5 Component test for queue-only mode (reload with a queue but no current episode): the bar renders with the neutral title, play is disabled, and the queue popover opens.
- [x] 5.6 App-level test: with no authenticated user the `PersistentPlayer` component is not mounted; setting a user mounts it; clearing the user stops playback (player store stopped state).
- [x] 5.7 Store test: a restored episode plays after reload — `togglePlay()` loads the element's source (`src`/`load()`) when the shared element has none.

## 6. Verification

- [x] 6.1 `pnpm test` and `pnpm build` in `frontend/`.
- [x] 6.2 Manual: play a list, open the Up Next panel, remove an item, skip next, go previous; reload the page and confirm the queue survives; finish the queue and confirm stop + empty panel; stop with episodes still queued — the bar must stay visible; clear the queue — the bar auto-hides after 10s.
- [x] 6.3 Manual: regression — single play (no list) still keeps any previously queued episodes.