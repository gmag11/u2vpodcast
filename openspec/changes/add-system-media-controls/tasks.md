## 1. Media Session Test Harness

- [x] 1.1 Extend the player-store test harness with deterministic `MediaSession`, `MediaMetadata`, action-handler, playback-state, and position-state doubles; verify tests can invoke registered actions and inspect published state without a real browser.
- [x] 1.2 Add baseline fallback tests for an absent Media Session API, a missing `setPositionState`, and an individually rejected action registration; verify ordinary playback succeeds and supported actions remain registered in every case.

## 2. System Action Integration

- [x] 2.1 Add lazy, idempotent, per-action Media Session registration beside the shared audio lifecycle in `frontend/src/stores/player.ts`; verify repeated playback creates no duplicate behavior and each unsupported registration is isolated.
- [x] 2.2 Route system play and pause through the existing player paths and reconcile `playing`, `loading`, and `stopped` from native audio events; verify pause persists progress and direct native play cannot leave `playing=true` with `stopped=true`.
- [x] 2.3 Route `nexttrack` to short `skipNext()` semantics and `previoustrack` to `playPrevious()` semantics; verify queue consumption, no listened marking, the three-second restart rule, history navigation, and empty-boundary no-ops in player-store tests.
- [x] 2.4 Route `seekforward`, `seekbackward`, and `seekto` through bounded original-timeline seek helpers; verify supplied offsets, the 15-second fallback, invalid-duration no-ops, duration clamping, and SponsorBlock rejected-interval adjustment.

## 3. Now-Playing Metadata and State

- [x] 3.1 Publish title, channel title, and optional artwork whenever the current episode changes, with a text-only fallback when artwork is absent or rejected; verify initial play and queue navigation replace stale metadata without interrupting playback.
- [x] 3.2 Synchronize Media Session playback state after play, pause, stop, completion, and source changes; verify tests observe `playing`, `paused`, and inactive states matching the shared player.
- [x] 3.3 Publish validated position state after metadata, time, seek, resume, speed, and episode transitions; verify duration and playback rate must be positive and finite, position is bounded, invalid values do not throw, and valid values match the shared audio element.

## 4. Authentication Teardown

- [x] 4.1 Add a dedicated player native-media teardown that flushes progress, stops and unloads the audio source, clears metadata, publishes inactive state, and unregisters system action handlers while retaining restorable queue/current-episode data; verify stale captured controls cannot restart playback after teardown and later playback registers a fresh session.
- [x] 4.2 Connect authentication loss in `frontend/src/App.vue` to the dedicated teardown; verify `App.test.ts` covers logout during active or paused playback and confirms the authenticated player can be established again after login.

## 5. Verification

- [x] 5.1 Run the focused player and app tests from the existing local frontend installation, then run the complete frontend test suite, type check, and production build without installing or updating packages; verify all commands pass or record any environment-only blocker without accessing the blocked npm registry.
- [ ] 5.2 Manually verify desktop system controls on the available representative Chromium browser plus any available Firefox/Safari environment; record play, pause, next, previous, relative/absolute seek availability, metadata, position, queue boundaries, and logout behavior for each tested browser/OS.
- [ ] 5.3 Manually verify mobile controls on available Android Chrome and iOS Safari or installed-PWA environments; record notification/lock-screen/headset behavior, metadata and artwork, background play/pause, next/previous, seek controls, queue boundaries, and logout behavior, explicitly distinguishing browser/OS display limitations from application failures.
