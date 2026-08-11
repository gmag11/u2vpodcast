## Context

The Vue 3 SPA (see archived `vue3-frontend-redesign`) renders episodes via `EpisodeCard.vue`, which currently owns a private `<audio>` element and local reactive state per card. Playback is scoped to one card, lost on navigation, and there is no persistent surface. The task is to make playback app-wide: one shared `<audio>` element, a persistent bottom bar, and full bidirectional synchronization with the per-card controls.

Media is served by the Rust backend at `/media/{slug}/{yt_id}.mp3` (range requests supported, verified 206). The design system provides coral/sky tokens, Phosphor icons, `glass-card`, and the `toHHMMSS` formatter. Auth is cookie-session; media requests succeed with the same session.

## Goals / Non-Goals

**Goals:**
- One global audio player store owning the single `<audio>` element and current episode.
- Persistent bottom player bar (full width, overlaying content) with thumbnail, title, play/pause, stop, scrubber, volume, and speed.
- Episode cards bound to the shared store; card and bar controls interchangeable.
- Playing a different episode swaps the shared source and retargets the bar.
- Auto-hide: after stop/end, the bar slides down and disappears 10 seconds later; reappears on play.
- Keep the media URL pattern, design tokens, icons, and auth unchanged.

**Non-Goals:**
- No backend/API changes; no new endpoints.
- No queueing/playlists, no next-track auto-advance, no seek bar preview/thumbnails, no background audio in another tab.
- No persistence of the stopped state across reloads (bar state is session-only).
- No visual redesign of the episode card beyond binding its controls to the shared store.

## Decisions

### D1: Audio player Pinia store owns the single `<audio>` element

Create `src/stores/player.ts`. The store creates and owns a module-scoped `HTMLAudioElement` (or creates it lazily once on first use) and exposes reactive state: `currentEpisode`, `playing`, `currentTime`, `duration`, `volume`, `muted`, `speed`, `loading`, and a `stopped` flag. All controls call store actions (`play(episode)`, `togglePlay()`, `pause()`, `stop()`, `seek(seconds)`, `setVolume()`, `toggleMute()`, `setSpeed()`). The element's events (`timeupdate`, `loadedmetadata`, `play`, `pause`, `waiting`, `canplay`, `ended`) update the store.

**Alternatives considered:** Keep a `<audio>` element in `App.vue` and pass via provide/inject — rejected, mixing DOM ownership with component tree; the store owning the element keeps one source of truth reachable from any component.

### D2: Store action `play(episode)` is the single entry point

`play(episode)` sets `currentEpisode`, builds `/media/{slug}/{yt_id}.mp3`, and:
- if the requested episode is already current and not stopped → just `audio.play()` (resume);
- otherwise swaps `audio.src`, reloads, and plays.

This makes both the card and the persistent bar call the same action, guaranteeing identical behavior and satisfying the "indistinguishable controls" requirement.

### D3: Persistent player component mounted in App.vue

Create `src/components/PersistentPlayer.vue`. `App.vue` renders `<PersistentPlayer />` after `<RouterView />` so it overlays every route. It is `position: fixed; bottom: 0; left: 0; right: 0`, `z-index` above page content but below the loading overlay, with `backdrop-blur` and `bg-surface` like the header.

### D4: Auto-hide with 10s delay and downward animation

The store exposes `stopped` (true after `stop()` or `ended`). The bar is hidden by default and on every stop: on `stopped`, a `setTimeout(10000)` is armed; if play/pause resumes before it fires, the timer is cleared. The bar uses Vue `<Transition name="slide-up-down">` — enter from `translateY(100%)` (slide up), leave to `translateY(100%)` (slide down). Visibility rules: hidden until the first `play()`; hidden when `stopped` and the 10 s window elapsed; visible while `playing` or `paused && !stopped` or during the 10 s post-stop window.

**Alternatives considered:** Hiding immediately on stop — rejected, the spec requires a visible 10s window; CSS-only animation without delay — rejected, the delay needs JS state; rendering the bar always-visible — rejected, spec requires it hidden by default until first playback.

### D5: EpisodeCard refactor binds to the store

`EpisodeCard.vue` drops its `<audio>` element and local audio state. Its computed/refs (`playing`, `currentTime`, `duration`, `volume`, `muted`, `speed`, `progress`, labels) become store getters. Its play button calls `store.play(episode)`; seek/volume/speed call the store actions. The card marks the currently-playing episode (highlight/active icon) by comparing `store.currentEpisode?.id`.

**Alternatives considered:** Keeping card-local audio and syncing — rejected, violates the single-source-of-truth requirement and risks dual audio.

### D6: Layout accounts for the fixed bar

Pages (`ChannelsView`, `EpisodesView`) get bottom padding (`pb-28` or an additive spacer) so the last card is not obscured by the persistent bar. This is a layout-only change; no logic change.

### D7: Stop vs pause semantics

`pause()` keeps position and `stopped=false`; `stop()` pauses, resets `currentTime=0`, sets `stopped=true`, and starts the hide timer. This is distinct so the bar auto-hides only on stop/end, not on ordinary pause.

## Risks / Trade-offs

- **Dual audio if a card keeps an old element** → The refactor removes `<audio>` from `EpisodeCard` entirely; the store is the only element owner.
- **Timer race on stop→play** → Clear the timeout inside the store/component whenever play is triggered; guard against firing after unmount.
- **Fixed bar covering content** → Compensate with page bottom padding; the bar is compact (single row) to minimize footprint.
- **Thumbnail 429s** → Unchanged from current behavior; the bar uses the same `episode.image` URL the card already uses (out of scope, being solved separately).
- **Element recreation** → The module-scoped element lives for the app session; route changes do not tear it down, so playback survives navigation.
