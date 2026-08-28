## Context

See `proposal.md` for motivation. `PlaylistView.vue` currently renders up/down buttons beside every episode and calls `playlists.reorder()` after each one-position move. The backend already accepts a complete ordered list through `PUT /api/1.0/playlist/reorder/`, and the Pinia playlist store updates its items after a successful response. The frontend uses Vue 3, Radix Vue dialog primitives, Phosphor icons, and Tailwind CSS.

Reordering must work with mouse, touch, and keyboard input. The playlist can also change while the drawer is open because playback completion removes playlist-sourced episodes asynchronously.

## Goals / Non-Goals

**Goals:**

- Keep drag interaction and draft order in a focused drawer component.
- Reuse the existing full-order API and keep the Pinia store as the shared confirmed playlist state.
- Support touch and keyboard operation without maintaining separate ordering algorithms.
- Recover cleanly from request failures and concurrent playlist changes.

**Non-Goals:**

- Changing the backend endpoint, database positions, or playlist lifecycle rules.
- Persisting a partially completed drag or batching several drops behind a save button.
- Reordering the player queue after it has already been seeded; a later playback start uses the newly stored playlist order.
- Adding drag and drop to episode lists outside the playlist reorder drawer.

## Decisions

### 1. Use a dedicated responsive drawer component

Create a playlist reorder component built with the existing Radix Vue dialog primitives. Its content is a right-side panel on wider screens and a bottom sheet on narrow screens, with focus trapping, Escape/overlay close behavior, a title, a close icon, and a scrollable list. On mobile, cap the sheet against the dynamic viewport (`dvh`), apply top and bottom safe-area insets, keep the header outside the independently scrolling list, and prevent horizontal overflow. `PlaylistView.vue` owns the open state and replaces each row's caret buttons with one reorder command in the page header, visible only for two or more items.

The drawer receives the store items and maintains a shallow local draft for drag rendering. Each row shows a drag-handle button with a minimum 44 by 44 CSS-pixel target, compact episode image, truncated title, and truncated channel title; full `EpisodeCard` controls are intentionally excluded. Apply `touch-action: none` only to the drag handle so vertical swipes elsewhere on the row continue to scroll the list.

**Alternatives considered:** Reusing `AppDialog.vue` unchanged would retain a centered modal rather than the requested drawer. Embedding drag handles in the main list would keep the page visually busy and would not satisfy the dedicated editing surface.

### 2. Use `vue-draggable-plus` for pointer and touch sorting

Add `vue-draggable-plus`, which integrates SortableJS with Vue reactive lists. Configure a dedicated handle, a short touch delay and movement tolerance that distinguish intentional dragging from scrolling, a dragged-item style, and a visible insertion/ghost style. Enable SortableJS auto-scroll on the drawer's list container with a bounded edge sensitivity and scroll speed tuned for the shorter mobile viewport. The nearest scrollable container moves continuously while the pointer remains in its top or bottom edge zone, stops outside that zone or at the corresponding boundary, and keeps the current drag active so off-screen insertion positions become reachable. On drag end, compare the resulting ID sequence with the pre-drag sequence; an unchanged sequence performs no request.

**Alternatives considered:** Native HTML Drag and Drop does not provide dependable touch behavior. A hand-built pointer-event implementation would add collision detection, scrolling, cancellation, and touch edge cases that a maintained library already handles.

### 3. Layer keyboard sorting on the same draft-order operation

The handle is focusable. Space or Enter picks up the focused row; Arrow Up and Arrow Down move it one insertion position in the same draft array; Space or Enter drops it; Escape cancels and restores its pre-pickup draft. A visually hidden live region announces pickup, current position, drop, cancellation, and persistence failure. Focus remains on the moved handle after each operation.

Pointer and keyboard paths both call one `commitOrder(ids)` function, so comparison, persistence, pending state, and rollback cannot diverge.

**Alternatives considered:** Keeping visible up/down buttons as the keyboard fallback would preserve two competing interfaces and would not provide the requested pick-up/drop model.

### 4. Persist every changed drop optimistically and serialize commits

Extend `playlists.reorder()` to snapshot `items`, apply the submitted ID order immediately, then call the existing API. On a failed or rejected request it restores the snapshot and returns the failure result. The drawer disables further drag and keyboard pickup while the request is pending, resets its draft from the store on failure, and emits a localized error notification. On success, the optimistic store order becomes the confirmed shared order.

Only one reorder commit is allowed at a time. Closing the drawer during a pending request is disabled so that feedback and focus remain coherent.

**Alternatives considered:** Waiting for the response before updating shared state makes the drop feel delayed and creates a temporary mismatch between drawer and main list. A save button would reduce requests but changes the requested drop-to-place interaction and introduces unsaved state.

### 5. Reconcile external playlist mutations while the drawer is idle

When no drag, keyboard pickup, or reorder request is active, watch the store's ordered IDs and refresh the drawer draft. At commit time, compare the draft's ID set with the current store ID set. If they differ, cancel the stale commit and reconcile from the store instead of sending an invalid complete order. This covers completion-based removal or other playlist updates while the drawer is open.

**Alternatives considered:** Freezing the playlist globally while the drawer is open would interfere with playback completion semantics. Always submitting the drawer snapshot could reintroduce a removed ID and fail server validation.

## Risks / Trade-offs

- **[Risk] The drag library increases the frontend bundle.** -> Import only its Vue composable/component path and verify the production build; its cross-input sorting behavior justifies the dependency.
- **[Risk] Scrollable drawers can fight touch dragging or auto-scroll too aggressively.** -> Require dragging from the handle, tune the touch threshold, edge sensitivity, and scroll speed, keep normal scrolling available from the rest of each row, and test both list boundaries.
- **[Risk] Optimistic rollback could overwrite a concurrent external mutation.** -> Serialize reorder commits and verify the current ID set before commit; on failure reload the playlist if the store ID set changed while the request was pending.
- **[Trade-off] Keyboard arrow movement is stepwise.** -> It remains deterministic and accessible while pointer/touch users can move directly to any insertion point.

## Migration Plan

1. Add the frontend dependency and drawer component, then switch `PlaylistView.vue` to the new reorder command.
2. Add optimistic rollback and pending-state handling to the playlist store without changing its public reorder payload.
3. Deploy as a frontend-only change; existing server versions remain compatible.
4. Roll back by restoring the caret controls and removing the drawer dependency. No data migration or API rollback is required.