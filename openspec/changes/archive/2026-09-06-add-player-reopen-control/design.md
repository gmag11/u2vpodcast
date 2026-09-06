## Context

The persistent player bar (`frontend/src/components/PersistentPlayer.vue`) is a single full-width element fixed to the bottom, wrapped in a `<Transition>` that slides it up/down. Its visibility is controlled by the `visible` ref, driven by a `watch` on `[playing, stopped, currentEpisode?.id, upNext.length]`:

- no episode and empty queue → hidden (nothing to restore)
- playing/paused (not stopped) → visible
- stopped → after 10 s the hide timer sets `visible = false`, whether or not an episode is loaded or the queue is non-empty (this change removes the queue-only branch that previously kept the bar visible)

Once hidden, the only way to get the bar back is to start playback again from an episode card or playlist. There is no way to inspect the stopped episode or the still-persisted up-next queue without launching audio.

The player state (current episode, queue, stopped flag) survives the hide; only the bar's rendered visibility toggles. This makes the reopen control purely presentational — no store change is required.

Today the "queue-only" state (a restored queue with no current episode) is an exception: the watch keeps the bar visible indefinitely so the queue stays reachable. Under this change that exception is removed — a queue-only stopped state auto-hides on the same 10 s timer — so the hidden bar is the single resting state for every stopped player, and the reopen control is the uniform way back.

## Goals / Non-Goals

**Goals:**
- Auto-hide the persistent bar in every stopped state, including queue-only (restored queue without a current episode), so it never stays visible while nothing plays.
- Provide a small floating control, rendered only while the bar is hidden and there is restorable content, that restores the bar without starting playback.
- Work in both compositions: wide (desktop, `sm:block`) and compact (mobile, `sm:hidden`, plus the expanded now-playing overlay).
- Keep the auto-hide behavior intact: reopening a stopped bar re-arms the 10 s timer, and the bar hides again as usual.
- Keep the control hidden when the bar is visible or there is nothing to restore.

**Non-Goals:**
- No new store actions or state. Reopening is `visible = true` only; no `play()`, `togglePlay()`, or queue mutation.
- No change to the auto-hide timer logic or its 10 s duration (the queue-only branch simply routes through the same timer instead of the "keep visible" exception).
- No separate route/page for a "mini player".

## Decisions

### Decision 1: The reopen control lives in `PersistentPlayer.vue` itself

Add a second fixed element, sibling to the existing bar `<Transition>`, at the bottom of the same component. It is shown when `!visible` and the player has restorable content (`player.currentEpisode != null || player.upNext.length > 0`).

- **Why**: visibility is already a component-local ref; the control's show/hide condition mirrors the bar's own `v-if`. Keeping it in the same component avoids a new store flag and extra prop drilling, and guarantees the control unmounts/remounts in lockstep with the bar across both compositions.
- **Why it covers queue-only**: `canRestore` is true when `upNext.length > 0`, so a hidden queue-only bar (no current episode) still shows the control — the only path back to the queue without playing.
- **Alternative considered**: a separate `PlayerReopenControl.vue` component mounted in `App.vue`. Rejected — it would need the same `visible` state duplicated or exposed, and both wide/compact rendering is already handled inside `PersistentPlayer`.

### Decision 2: Stopped visibility is uniform — the queue-only branch routes through the auto-hide timer

The watch currently has a branch that keeps the bar visible when `episodeId == null` but the queue is non-empty. This change removes that exception: a stopped bar with no current episode (queue-only) now runs `armHideTimer()` exactly like a stopped bar with an episode loaded, so it hides after 10 s. The `episodeId == null && queueLength === 0` branch still hides immediately (nothing to restore). A stopped bar with no current episode and a non-empty queue therefore becomes hidden after its delay, and the reopen control (Decision 1) is what brings it back.

- **Why**: a bar that stays visible while nothing plays contradicts the "auto-hide on stop" model and the reason this control exists. With the exception removed there is exactly one hidden resting state for every stopped player, so the reopen control is the single, predictable way to restore the bar.
- **Alternative considered**: keep the queue-only exception and only add the control for the stopped-with-episode case. Rejected — the user wants a loaded-but-stopped queue to be hidden on page load and restorable only via the control, so the exception must go.

### Decision 3: Activation is `visible = true` only

Clicking the control sets `visible.value = true`. The existing `watch` on player state fires whenever the player state changes (not on manual visibility), so re-showing does not need to emulate a play action. To let the auto-hide timer behave consistently when reopening a *stopped* bar, activation also calls `clearHideTimer()` then `armHideTimer()` when the player is stopped, mirroring the branch the watch takes for a freshly stopped state.

- **Why**: keeps the "reopened stopped bar still auto-hides after 10 s" scenario true without touching the store. This holds for both a reopened stopped bar with an episode loaded and a reopened queue-only bar: activation arms the timer, and the bar hides again after the delay.
- **Alternative considered**: a store action `revealPlayer()`. Rejected — there is no shared state to change; the store does not own `visible`.

### Decision 4: Positioning and presentation

Render a single discrete drawer tab anchored to the bottom edge, centered horizontally, replacing the visual slot the bar occupies when shown. It carries only a caret-up glyph (no text label) so it reads as "expand the player" without competing with page content:

- A small rounded-top tab pinned at the screen bottom (`fixed bottom-0`, `left-1/2` + `-translate-x-1/2`, `rounded-t-xl`), with `border-outline` top corners and `bg-surface-card`, matching where the hidden bar would sit.
- Consistent across both compositions (wide and compact): same centered tab, no responsive reflow.
- The icon is a re-used chevron glyph `PhCaretUp`. It uses existing design tokens (`bg-surface-card`, `border-outline`, `shadow-card`) and sits at `z-30`, matching the bar. An `aria-label` from i18n describes the action ("Show player") for assistive tech.

- **Why**: a minimal, always-visible affordance reads clearly as "the player is collapsed here" and never overlaps content — unlike a floating right-corner pill it stays on the same bottom edge the bar owns, so it is unobtrusive yet obviously interactive.
- **Why no text**: the user asked for a discreet control without text; the caret-up glyph plus the same bottom-edge position is enough to signal the collapsed player. Wide and compact get identical affordances, so a single element (no `sm:` variants) is enough.

### Decision 5: Data-testid for the control

Expose a stable `data-testid="player-reopen"` so component tests can target it (mirroring existing `data-testid="player-wide"`, `player-compact`, `player-expanded`). The bar element also carries `data-testid="player-bar"` so tests distinguish it from the reopen tab (both sit at `fixed bottom-0`).

## Risks / Trade-offs

- **[Risk] Control may cover content or overlap other bottom controls** → Keep it small and use the same `z-30` stacking as the bar; it only appears when the bar is gone, so collision with bar controls is impossible.
- **[Risk] Auto-hide timer re-arms on every stop-state watch hit** → Acceptable and already the behavior; reopening a stopped bar simply starts one fresh 10 s countdown.
- **[Risk] Duplicated show/hide condition between the control and the bar `v-if`** → Derive a single computed `canRestore` (`currentEpisode != null || upNext.length > 0`) used by both the control and, where relevant, the bar's conditional rendering.
- **[Risk] Removing the queue-only exception changes behavior for users who relied on the bar staying visible to reach a loaded queue** → Intentional under this change: the reopen control is the replacement path, so the queue stays one tap away without auto-playing. Covered by a dedicated queue-only test.
- **[Risk] A cold page load with a restored stopped queue briefly shows nothing where the bar used to be** → Expected: the bar is hidden by default when stopped and the reopen control appears instead, which is the requested behavior.

## Migration Plan

Frontend-only behavior change. Ship behind the normal frontend build; no backend, DB, or API migration. Rollback is reverting the component change (which restores the queue-only visible exception).

## Open Questions

None.
