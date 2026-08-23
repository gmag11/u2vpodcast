## 1. Queue store API

- [ ] 1.1 In `frontend/src/stores/player.ts` make `upNext` public state and add a private `playStack: Episode[]` for the previous control.
- [ ] 1.2 Add mutations: `removeFromQueue(episodeId)`, `clearQueue()`, `skipNext()` (shift + play), `playPrevious()` (dual: when `currentTime > 3s` restart the current episode via `seek(0)`; otherwise pop `playStack` and play without re-seeding), and seed on play: when `list` provided replace `upNext`; when omitted keep it.
- [ ] 1.3 Update `advance()` from step 1: shift, push the finished episode onto `playStack`, play, persist; on empty queue stop and clear.
- [ ] 1.4 Keep `playStack` bounded (e.g. keep last 50) to avoid unbounded growth in long sessions.

## 2. Persistence helper

- [ ] 2.1 Create `frontend/src/lib/utils/queue.storage.ts` with `saveQueue(payload)` and `loadQueue()` using `localStorage` key `u2vpodcast.up-next.v1`; wrap serialization/parsing in try/catch and return `null`/defaults on failure.
- [ ] 2.2 Combine `upNext` + `playStack` in the stored payload and rehydrate both in the store setup; persist after every mutation.
- [ ] 2.3 Add tests for the storage round-trip and malformed-payload handling.

## 3. Persistent player UI

- [ ] 3.1 In `frontend/src/components/PersistentPlayer.vue` add previous and next buttons; disable next when `upNext` is empty; keep previous enabled while an episode is loaded (it can always restart the current episode) and only disable it when nothing is loaded; wire to `playPrevious`/`skipNext`.
- [ ] 3.2 Add a queue toggle button (e.g. `PhList` icon) opening an "Up next" popover (radix-vue `DropdownMenuRoot` or similar) listing `upNext` with thumbnail, title, channel, count, per-item remove (→ `removeFromQueue`) and a "clear all" (→ `clearQueue`).
- [ ] 3.3 Show an empty-state line in the popover when the queue is empty (i18n string in en/es).

## 4. Seed wiring (reuse step 1)

- [ ] 4.1 Confirm `EpisodesView.vue` and `HistoryView.vue` still pass `:list="filteredEpisodes"`; adjust only if behavior changed around orphan-play queue keeping.

## 5. Tests

- [ ] 5.1 Extend `frontend/src/stores/player.test.ts`: seed replaces queue; play without list keeps queue; `removeFromQueue`/`clearQueue`; `skipNext`; `playPrevious` dual behavior (currentTime > 3s → restart at 0, ≤ 3s → pop `playStack`); advance pushes to `playStack`; stop clears and persists empty.
- [ ] 5.2 Test that `saveQueue`/`loadQueue` tolerate corrupt data and that rehydration restores a previously saved queue.
- [ ] 5.3 Component test for the bar: next/prev disabled states and popover rendering (per `AppHeader.test.ts` pattern with `@vue/test-utils`).

## 6. Verification

- [ ] 6.1 `pnpm test` and `pnpm build` in `frontend/`.
- [ ] 6.2 Manual: play a list, open the Up Next panel, remove an item, skip next, go previous; reload the page and confirm the queue survives; finish the queue and confirm stop + empty panel.
- [ ] 6.3 Manual: regression — single play (no list) still keeps any previously queued episodes.