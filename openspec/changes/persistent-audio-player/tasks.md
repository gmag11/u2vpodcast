## 1. Shared audio player store

- [x] 1.1 Create `src/stores/player.ts` (Pinia) that owns a single module-scoped `HTMLAudioElement` created lazily once, and exposes reactive state: `currentEpisode`, `playing`, `currentTime`, `duration`, `volume`, `muted`, `speed`, `loading`, `stopped`
- [x] 1.2 Implement store actions: `play(episode)` (single entry point that swaps `audio.src` to `/media/{slug}/{yt_id}.mp3` when the episode differs and plays), `togglePlay()`, `pause()`, `stop()` (pause + reset time + set `stopped`), `seek(seconds)`, `setVolume(v)`, `toggleMute()`, `setSpeed(s)`
- [x] 1.3 Wire the element's events into the store: `timeupdate`, `loadedmetadata`, `play`, `pause`, `waiting`, `canplay`, `ended` (ended → treat as stop)
- [x] 1.4 Expose computed getters: `progress` (%), `currentLabel`, `durationLabel`, `isCurrent(episode)` helper

## 2. Persistent player bar

- [x] 2.1 Create `src/components/PersistentPlayer.vue`: fixed bottom, full width, overlaying content, `backdrop-blur` + `bg-surface`, showing thumbnail, title, play/pause, stop, position scrubber, volume (mute + range), and speed menu (0.5x–2x), all bound to the player store
- [x] 2.2 Implement visibility and animations: bar hidden by default; slides up (translateY(100%) → 0) on first play and on every new play; when `stopped` becomes true keep visible 10 s (setTimeout, cleared on any play/pause) then hide with a downward slide-out (translateY(100%)); use a Vue `<Transition>` handling both enter (slide up) and leave (slide down)
- [x] 2.3 Mount `<PersistentPlayer />` in `App.vue` after `<RouterView />` so it overlays every route

## 3. Refactor EpisodeCard

- [x] 3.1 Remove the `<audio>` element and all local audio state from `EpisodeCard.vue`; bind play/pause, seek, volume, and speed controls to the player store actions
- [x] 3.2 Show active state on the card when `player.isCurrent(episode)` (playing indicator), and make the card play button call `player.play(episode)`
- [x] 3.3 Reuse the existing design tokens/icons; keep the card's thumbnail, title, description, and YouTube link unchanged

## 4. Layout and verification

- [x] 4.1 Add bottom padding to `ChannelsView` and `EpisodesView` (`pb-28`) so content is not obscured by the fixed bar
- [x] 4.2 Run `pnpm lint`, `pnpm check` (build), and `pnpm test`; verify build passes
- [x] 4.3 Smoke-test: play from a card → persistent bar appears; toggle pause from bar → card reflects it; play another episode → bar retargets; stop → bar auto-hides after 10 s; speed/volume/seek work from both card and bar
