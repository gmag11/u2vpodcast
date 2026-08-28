## Context

See `proposal.md` for motivation. `PlaylistView.vue` currently wraps every `EpisodeCard` in a row with two stacked caret buttons that call `playlists.reorder()` after each one-position move. The desired interaction replaces that highlighted control column with the compact six-dot grip shown in the reference, while leaving the cards in the main playlist rather than duplicating them in a drawer. The backend already accepts a complete ordered list through `PUT /api/1.0/playlist/reorder/`.

Reordering must work with mouse, touch, and keyboard input. Playback completion can remove a playlist item while an interaction is active.

## Goals / Non-Goals

**Goals:**

- Replace the two caret buttons with one visually restrained handle beside each existing card.
- Move complete playlist rows directly in the main view with clear insertion feedback.
- Preserve card actions and ordinary page scrolling outside the handle.
- Reuse the existing full-order API and recover cleanly from failures or concurrent removals.

**Non-Goals:**

- Opening a drawer, modal, bottom sheet, or separate reorder mode.
- Redesigning or duplicating the episode card content.
- Changing the backend endpoint, database positions, or playlist lifecycle rules.
- Reordering a player queue that has already been seeded.
- Adding drag and drop to episode lists outside the playlist view.

## Decisions

### 1. Make each existing playlist wrapper the draggable row

Keep `EpisodeCard` unchanged and make its outer wrapper in `PlaylistView.vue` the sortable item. Replace the caret column with one handle button using a six-dot grip icon. The handle sits immediately left of the card, is vertically centered, has a muted default treatment, and gains clear hover, focus, active, and disabled states. It is shown only when at least two items can be reordered.

This keeps playback, favorite, playlist removal, links, and all card-responsive behavior in their current component. The active drag style applies to the complete wrapper so the user sees the card itself move, while a gap or marker between cards communicates the drop position.

**Alternatives considered:** A dedicated drawer was rejected because it duplicates the playlist and adds an unnecessary mode. Making the whole card draggable was rejected because it conflicts with the card's buttons, links, and mobile scrolling.

### 2. Use `vue-draggable-plus` with a handle-only activation area

Bind the ordered wrapper list to `vue-draggable-plus`/SortableJS and configure the handle selector so dragging cannot start from card content. Use a short touch delay and movement tolerance to distinguish an intentional handle drag from page scrolling. Apply drag-specific `touch-action` only to the handle, which has a minimum 44 by 44 CSS-pixel target on mobile; keep the visual grip smaller so it does not dominate the card or force horizontal overflow.

On drag start, snapshot the current IDs. On drag end, compare the resulting sequence with that snapshot and commit only a changed order. The insertion ghost and chosen row remain visible in both light and dark themes.

**Alternatives considered:** Native HTML Drag and Drop has unreliable touch behavior. A hand-built pointer implementation would duplicate collision, cancellation, and scrolling behavior supplied by SortableJS.

### 3. Auto-scroll the page during long-distance dragging

Enable SortableJS auto-scroll against the nearest scrollable ancestor/window. When the pointer remains within a bounded zone at the top or bottom of the visible viewport, scroll continuously in that direction while preserving the active drag. Tune edge sensitivity and speed for short mobile viewports, and stop at the document boundary or when the pointer leaves the edge zone.

**Alternatives considered:** A fixed-height inner playlist scroller would create nested scrolling in the main page. Requiring repeated short drops would retain the inefficiency this change is intended to remove.

### 4. Layer keyboard sorting on the same order operation

The handle is focusable. Space or Enter picks up the focused row; Arrow Up and Arrow Down move it one insertion position; Space or Enter drops it; Escape restores its pre-pickup order. A visually hidden live region announces pickup, current position, drop, cancellation, and persistence failure. Focus follows the moved handle.

Pointer and keyboard paths call one commit operation so comparison, persistence, pending state, and rollback do not diverge.

**Alternatives considered:** Retaining the visible caret buttons for keyboard users would preserve the clutter and create two competing reorder interfaces.

### 5. Persist changed drops optimistically and reconcile external changes

Extend `playlists.reorder()` to snapshot `items`, apply a valid submitted ID order immediately, and call the existing API. Serialize commits. On failure, restore the snapshot unless the playlist's ID set changed concurrently; in that case reload the authoritative playlist. Disable all handles while persistence is pending and show a localized error notification on rollback.

Before committing, compare the dragged ID set with the current store ID set. If playback completion or another action removed an item, cancel the stale commit and reconcile the rendered order from the store instead of submitting an invalid complete order.

**Alternatives considered:** Waiting for the server before updating the list makes the completed drop feel delayed. Submitting a stale snapshot can reintroduce a removed ID or fail server validation.

## Risks / Trade-offs

- **[Risk] Touch dragging can conflict with page scrolling.** -> Start drag only from the handle, use touch delay/tolerance, and leave the card surface available for ordinary scrolling.
- **[Risk] Viewport auto-scroll can be too aggressive.** -> Bound and tune its edge zone and speed, and test both directions on short mobile viewports.
- **[Risk] The added handle can squeeze cards on narrow screens.** -> Keep one compact visual grip inside a 44-pixel target and allow the card wrapper to shrink without horizontal overflow.
- **[Risk] Optimistic rollback can overwrite a concurrent removal.** -> Serialize commits, compare ID sets, and reload when the set changes during the request.

## Migration Plan

1. Add the drag dependency and optimistic store behavior.
2. Replace the existing caret controls in `PlaylistView.vue` with inline sortable handles and remove the old `move()` path.
3. Deploy as a frontend-only change; existing server versions remain compatible.
4. Roll back by restoring the caret controls and removing the drag dependency. No data or API migration is required.