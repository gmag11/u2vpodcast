## Context

The persistent player bar (`frontend/src/components/PersistentPlayer.vue`) is a single full-width element fixed to the bottom, wrapped in a `<Transition>` that slides it up/down. Its visibility is controlled by the `visible` ref, driven by a `watch` on `[playing, stopped, currentEpisode?.id, upNext.length]`:

- no episode and empty queue → hidden
- playing/paused (not stopped) → visible
- queue-only mode (restored queue without a current episode) → visible
- stopped → after 10 s the hide timer sets `visible = false`

Once hidden, the only way to get the bar back is to start playback again from an episode card or playlist. There is no way to inspect the stopped episode or the still-persisted up-next queue without launching audio.

The player state (current episode, queue, stopped flag) survives the hide; only the bar's rendered visibility toggles. This makes the reopen control purely presentational — no store change is required.

## Goals / Non-Goals

**Goals:**
- Provide a small floating control, rendered only while the bar is hidden, that restores the bar without starting playback.
- Work in both compositions: wide (desktop, `sm:block`) and compact (mobile, `sm:hidden`, plus the expanded now-playing overlay).
- Keep the auto-hide behavior intact: reopening a stopped bar re-arms the 10 s timer, and the bar hides again as usual.
- Keep the control hidden when the bar is visible or there is nothing to restore.

**Non-Goals:**
- No new store actions or state. Reopening is `visible = true` only; no `play()`, `togglePlay()`, or queue mutation.
- No change to the auto-hide timer logic or its 10 s duration.
- No separate route/page for a "mini player".

## Decisions

### Decision 1: The reopen control lives in `PersistentPlayer.vue` itself

Add a second fixed element, sibling to the existing bar `<Transition>`, at the bottom of the same component. It is shown when `!visible` and the player has restorable content (`player.currentEpisode != null || player.upNext.length > 0`).

- **Why**: visibility is already a component-local ref; the control's show/hide condition mirrors the bar's own `v-if`. Keeping it in the same component avoids a new store flag and extra prop drilling, and guarantees the control unmounts/remounts in lockstep with the bar across both compositions.
- **Alternative considered**: a separate `PlayerReopenControl.vue` component mounted in `App.vue`. Rejected — it would need the same `visible` state duplicated or exposed, and both wide/compact rendering is already handled inside `PersistentPlayer`.

### Decision 2: Activation is `visible = true` only

Clicking the control sets `visible.value = true`. The existing `watch` on player state fires whenever the player state changes (not on manual visibility), so re-showing does not need to emulate a play action. To let the auto-hide timer behave consistently when reopening a *stopped* bar, activation also calls `clearHideTimer()` then `armHideTimer()` when the player is stopped, mirroring the branch the watch takes for a freshly stopped state.

- **Why**: keeps the "reopened stopped bar still auto-hides after 10 s" scenario true without touching the store.
- **Alternative considered**: a store action `revealPlayer()`. Rejected — there is no shared state to change; the store does not own `visible`.

### Decision 3: Positioning and presentation

Render a single compact circular button, fixed near the bottom edge, centered or right-aligned depending on composition:

- Desktop: a pill anchored bottom-center/right above where the bar would sit.
- Mobile compact: bottom-right above the tab bar / safe area.

The icon is a re-used play/chevron glyph (e.g. `PhCaretUp`) to signal "expand the player". It uses existing design tokens (`bg-surface-card`, `border-outline`, `shadow-card`) and sits at `z-30`, matching the bar. A `title`/`aria-label` from i18n describes the action ("Show player").

- **Why**: the existing design system already defines the surface/shadow tokens; no new styling primitives needed.

### Decision 4: Data-testid for the control

Expose a stable `data-testid="player-reopen"` so component tests can target it (mirroring existing `data-testid="player-wide"`, `player-compact`, `player-expanded`).

## Risks / Trade-offs

- **[Risk] Control may cover content or overlap other bottom controls** → Keep it small and use the same `z-30` stacking as the bar; it only appears when the bar is gone, so collision with bar controls is impossible.
- **[Risk] Auto-hide timer re-arms on every stop-state watch hit** → Acceptable and already the behavior; reopening a stopped bar simply starts one fresh 10 s countdown.
- **[Risk] Duplicated show/hide condition between the control and the bar `v-if`** → Derive a single computed `canRestore` (`currentEpisode != null || upNext.length > 0`) used by both the control and, where relevant, the bar's conditional rendering.

## Migration Plan

Frontend-only additive change. Ship behind the normal frontend build; no backend, DB, or API migration. Rollback is reverting the component change.

## Open Questions

None.
