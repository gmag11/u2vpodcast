## 1. Player store queue

- [x] 1.1 In `frontend/src/stores/player.ts` add `upNext` queue state (`ref<Episode[]>([])`) and extend `play(episode: Episode, list?: Episode[])`: when `list` is provided, seed `upNext` with `list.slice(list.findIndex(e => e.id === episode.id) + 1)`; when omitted, leave `upNext` empty.
- [x] 1.2 Add `advance()`: shift the first item of `upNext`; if present, `await play(next)`; otherwise call the existing stop behavior exactly as `onEnded` does today.
- [x] 1.3 Replace `onEnded` body: set `playing=false` and call `advance()` instead of `stop()`.

## 2. Episode card context prop

- [ ] 2.1 In `frontend/src/components/EpisodeCard.vue` add optional prop `list?: Episode[]` (default `undefined`).
- [ ] 2.2 Change both play buttons (`sm:hidden` block and desktop block) to call `player.play(props.episode, props.list)`.

## 3. View wiring

- [ ] 3.1 In `frontend/src/views/EpisodesView.vue` pass `:list="filteredEpisodes"` to `EpisodeCard`.
- [ ] 3.2 In `frontend/src/views/HistoryView.vue` pass `:list="filteredEpisodes"` to `EpisodeCard`.

## 4. Tests

- [x] 4.1 Create `frontend/src/stores/player.test.ts` (Vitest) covering: play with a list seeds the queue from the index after the episode; play without a list leaves the queue empty; `advance` plays the next and drains the queue; `advance` on an empty queue calls stop (position reset, playing false).
- [x] 4.2 Mock `HTMLAudioElement` (jsdom via existing `vitest.config.ts`) so the store's `ensureAudio` wiring works in tests.

## 5. Verification

- [x] 5.1 Run `pnpm test` in `frontend/` and confirm the new player tests pass.
- [x] 5.2 Run `pnpm build` in `frontend/` and confirm no type/build errors.
- [ ] 5.3 Manual: open a channel's episodes, press play on a middle episode, let it finish (or seek near the end) and confirm the next visible episode starts; repeat on the History screen with an active search filter; confirm the last episode stops instead of throwing.
- [ ] 5.4 Manual: confirm play from a card while another episode is playing still swaps source as before (regression check).
- [ ] 5.5 Manual: with a queue active, the next button in the player bar (right of stop) advances; with an empty queue the button appears disabled and does nothing.

## 6. Player bar next control

- [x] 6.1 In `frontend/src/components/PersistentPlayer.vue` add a next button (`PhSkipForward`) to the right of the stop button, disabled when `player.upNext` is empty, calling `player.advance()`; add the `player.next` i18n string in en/es.
- [x] 6.2 Add a component test (`frontend/src/components/PersistentPlayer.test.ts`): with a queued episode the next button is enabled and clicking it plays the next episode; with an empty queue the button is disabled.