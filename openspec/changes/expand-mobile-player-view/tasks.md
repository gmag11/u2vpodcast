## 1. Mobile mode mapping logic

- [ ] 1.1 Implement a pure `mobilePlaybackMode` derivation (`'normal' | 'repeat' | 'shuffle'`) from the store's `shuffle`/`repeat` state and a `cycleMobilePlaybackMode()` helper that advances to the next state via the existing `player.toggleShuffle()`/`player.cycleRepeat()` (or direct state setters); verify with unit tests covering all documented cycles (normal→repeat→shuffle→normal) and the "closest state" fallback for repeat-one and shuffle+repeat combinations.

## 2. Expanded view component

- [ ] 2.1 Create `PersistentPlayerExpanded.vue` with the chevron-down close control, large thumbnail, title, channel name, and a `Transition` sliding up from the bottom, hidden by default; verify it renders when mounted with `expanded=true` and emits a close event on chevron press.
- [ ] 2.2 Add an interactive progress bar with elapsed and remaining/total time labels, wired to `player.seek`/`player.progress`/`player.currentLabel`/`player.durationLabel`; verify a test that simulates a click/drag at a given offset calls `player.seek` with the expected time and that labels update as `player.currentTime` changes.
- [ ] 2.3 Add the playback speed control (presets + +/- stepper) reusing `player.setSpeed`, `SPEED_MIN`, `SPEED_MAX`, `SPEED_STEP`; verify with a test that preset buttons and stepper buttons call `player.setSpeed` with the expected values and respect min/max disabling.
- [ ] 2.4 Add the combined shuffle/repeat control using the mapping from Task 1.1, visually indicating the active of the three states; verify with a test that pressing it cycles the underlying store state as specified.
- [ ] 2.5 Add the "Up next" queue toggle opening the existing queue panel markup/behavior (list, remove, clear); verify with a test that opening it lists `player.upNext` and remove/clear actions call the corresponding store methods.
- [ ] 2.6 Add transport controls: previous, seek-back-10s, play/pause, seek-forward-10s, next, reusing `player.playPrevious`, `player.togglePlay`, `player.skipNext`, and new/existing relative-seek helpers for ±10s; verify with tests that each button calls the expected store method/argument.
- [ ] 2.7 Verify no volume/mute control exists anywhere in `PersistentPlayerExpanded.vue` (assert absence in a rendering test).

## 3. Wiring into the compact bar

- [ ] 3.1 Add a tap handler on the compact bar's thumbnail (`data-testid="player-compact"` block) that sets `expanded.value = true`, only active while the compact composition is shown; verify with a test simulating a click on the thumbnail while the viewport is narrow.
- [ ] 3.2 Mount `PersistentPlayerExpanded.vue` from `PersistentPlayer.vue` bound to `expanded`, passing/reading the shared player store; verify the expanded view appears over page content and playback is uninterrupted (position/playing state unchanged) across the transition.
- [ ] 3.3 Add a breakpoint watcher that force-closes the expanded view when the viewport reaches >= 640px; verify with a test that resizing (or mocking `matchMedia`) while expanded triggers `expanded.value = false`.

## 4. Localization and accessibility

- [ ] 4.1 Add any new i18n keys needed for the expanded view (aria-labels for close, seek, transport controls not already covered by existing `player.*` keys) to the locale files; verify the app builds with no missing-key warnings.
- [ ] 4.2 Verify keyboard and assistive-technology operability of the new interactive scrubber (role="slider", aria-valuenow/min/max) and all buttons (aria-label present) via component tests.

## 5. Regression checks

- [ ] 5.1 Run the existing `PersistentPlayer.test.ts` suite and confirm the compact bar's existing behavior (auto-hide, progress track read-only state, title scrolling) is unchanged aside from the added tap-to-expand affordance.
- [ ] 5.2 Add/confirm a test that the wide composition (>=640px) is entirely unaffected by this change (same controls, same markup, no expanded view reachable).
