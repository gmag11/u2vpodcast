## 1. Component: reopen control

- [ ] 1.1 In `PersistentPlayer.vue`, add a `canRestore` computed (`player.currentEpisode != null || player.upNext.length > 0`) and use it in the bar's existing `v-if` condition.
- [ ] 1.2 Add a `restoreBar()` handler that sets `visible.value = true` and, when the player is stopped, calls `clearHideTimer()` then `armHideTimer()` so the reopened stopped bar still auto-hides after 10 s.
- [ ] 1.3 Add a `revealed`-style visibility derivation (or reuse `visible`) so the control renders only while the bar is hidden.
- [ ] 1.4 Render a floating button (sibling of the bar `<Transition>` in the same template) shown when `!visible && canRestore`, with `data-testid="player-reopen"`, an `aria-label` bound to a new i18n key, and the caret-up icon.
- [ ] 1.5 Place the control consistently for both compositions: wide (bottom-center/right, `z-30`) and compact (bottom-right safe-area), using existing design tokens.

## 2. i18n

- [ ] 2.1 Add `player.showPlayer` ("Show player" / "Mostrar reproductor") to `frontend/src/i18n/locales/en.json` and `es.json`.

## 3. Tests

- [ ] 3.1 Component test: the reopen control is absent while the bar is visible.
- [ ] 3.2 Component test: after the bar hides (auto-hide elapsed), the reopen control appears.
- [ ] 3.3 Component test: activating the reopen control restores the bar without setting `playing` true or mutating the queue/current episode.
- [ ] 3.4 Component test: reopening a stopped bar re-arms the hide timer and the bar hides again after 10 s, with the control reappearing.
- [ ] 3.5 Component test: no reopen control when there is no current episode and no queue to restore.
- [ ] 3.6 Update the existing queue-non-empty hide test if the added element changes the `fixed.bottom-0` selector assumptions.

## 4. Spec sync and verification

- [ ] 4.1 Run `frontend` unit tests (`vitest run`) and typecheck (`vue-tsc -b`), ensure all green.
- [ ] 4.2 Run lint/format (`eslint`, `prettier`) on touched files.
- [ ] 4.3 Validate the change with `openspec validate --change add-player-reopen-control`.
