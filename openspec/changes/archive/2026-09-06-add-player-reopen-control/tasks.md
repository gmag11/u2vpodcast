## 1. Visibility: uniform auto-hide for every stopped state

- [x] 1.1 In `PersistentPlayer.vue`, remove the queue-only exception in the visibility `watch` that kept the bar visible when `episodeId == null` and the queue is non-empty; that state now runs `armHideTimer()` like any other stop, so a queue-only stopped bar hides after 10 s.
- [x] 1.2 Keep the `episodeId == null && queueLength === 0` branch hiding immediately (nothing to restore) with `clearHideTimer()`.
- [x] 1.3 Verify the initial state on a cold page load with a restored stopped queue (no current episode): `visible` starts false and no hide timer fires while nothing is playing, so the bar is hidden and the reopen control is the path back.

## 2. Component: reopen control

- [x] 2.1 In `PersistentPlayer.vue`, add a `canRestore` computed (`player.currentEpisode != null || player.upNext.length > 0`) and use it in the bar's existing `v-if` condition.
- [x] 2.2 Add a `restoreBar()` handler that sets `visible.value = true` and, when the player is stopped, calls `clearHideTimer()` then `armHideTimer()` so the reopened stopped bar still auto-hides after 10 s. This applies to both a stopped bar with an episode loaded and a queue-only bar.
- [x] 2.3 Add a `revealed`-style visibility derivation (or reuse `visible`) so the control renders only while the bar is hidden.
- [x] 2.4 Render a floating button (sibling of the bar `<Transition>` in the same template) shown when `!visible && canRestore`, with `data-testid="player-reopen"`, an `aria-label` bound to a new i18n key, and the caret-up icon.
- [x] 2.5 Place the control consistently for both compositions: wide (bottom-center/right, `z-30`) and compact (bottom-right safe-area), using existing design tokens.

## 3. i18n

- [x] 3.1 Add `player.showPlayer` ("Show player" / "Mostrar reproductor") to `frontend/src/i18n/locales/en.json` and `es.json`.

## 4. Tests

- [x] 4.1 Component test: the reopen control is absent while the bar is visible.
- [x] 4.2 Component test: after the bar hides (auto-hide elapsed), the reopen control appears.
- [x] 4.3 Component test: activating the reopen control restores the bar without setting `playing` true or mutating the queue/current episode.
- [x] 4.4 Component test: reopening a stopped bar re-arms the hide timer and the bar hides again after 10 s, with the control reappearing.
- [x] 4.5 Component test: no reopen control when there is no current episode and no queue to restore.
- [x] 4.6 Component test: a stopped queue-only state (no current episode, non-empty queue) auto-hides after 10 s and shows the reopen control; activating it restores the bar with the queue intact without playing.
- [x] 4.7 Component test: a cold page load with a restored stopped queue but no current episode renders the bar hidden and the reopen control visible.
- [x] 4.8 Update the existing queue-non-empty hide test if the added element changes the `fixed.bottom-0` selector assumptions.

## 5. Spec sync and verification

- [x] 5.1 Run `frontend` unit tests (`vitest run`) and typecheck (`vue-tsc -b`), ensure all green.
- [x] 5.2 Run lint/format (`eslint`, `prettier`) on touched files.
- [x] 5.3 Validate the change with `openspec validate --change add-player-reopen-control`.
