## 1. Drag-and-Drop Foundation

- [ ] 1.1 Add `vue-draggable-plus` to the frontend dependencies and verify `pnpm install --frozen-lockfile` succeeds with the updated lockfile.
- [ ] 1.2 Add localized English and Spanish strings for opening and closing the reorder drawer, drag-handle instructions, pickup/move/drop/cancel announcements, pending state, and reorder failure; verify both locale files expose the same playlist keys.

## 2. Playlist Store Persistence

- [ ] 2.1 Update the playlist store reorder action to apply a valid complete ID order optimistically, serialize reorder requests, and retain a snapshot for rollback; verify store tests cover the immediate optimistic order and successful persistence payload.
- [ ] 2.2 Restore or reload the latest confirmed playlist after reorder failure without overwriting a concurrent removal, and verify store tests cover ordinary rollback plus an item removed while the request is pending.

## 3. Reorder Drawer

- [ ] 3.1 Create a responsive Radix Vue reorder drawer that becomes a dynamic-viewport bottom sheet on mobile, respects safe-area insets, keeps its header fixed above an independently scrolling list, prevents horizontal overflow, and uses compact truncated rows with 44 by 44 CSS-pixel controls; verify component tests cover rendering, responsive classes, and close behavior.
- [ ] 3.2 Bind the drawer draft to `vue-draggable-plus` through a dedicated handle, configure touch delay/tolerance and handle-only touch-action so row swipes still scroll, style the active row and insertion ghost, and configure edge-triggered auto-scroll on the list container so a drag can reach positions above or below the visible area; verify component tests cover ordinary touch scrolling, scrolling at both drag edges, stopping outside the edge zone and at list boundaries, the full ID payload, and unchanged-drop no-op behavior.
- [ ] 3.3 Implement keyboard pickup, arrow movement, drop, and cancellation on the same draft-order path, retaining handle focus and announcing state through a live region; verify component tests exercise the full keyboard sequence and announcements.
- [ ] 3.4 Disable interaction and closing while persistence is pending, reconcile idle drafts with external store mutations, reject stale-ID drops, and show a localized failure notification after rollback; verify component tests cover pending and concurrent-removal cases.

## 4. Playlist View Integration

- [ ] 4.1 Replace the per-row caret controls in `PlaylistView.vue` with a header reorder command that appears only for two or more episodes and opens the drawer; verify a view test covers command visibility, drawer opening, and the absence of stepwise controls.
- [ ] 4.2 Confirm play-all, episode playback, playlist removal, and completion-driven removal still use the store's current order after a reorder; verify the focused playlist view and player tests pass.

## 5. Verification

- [ ] 5.1 Run `pnpm test`, `pnpm typecheck`, `pnpm lint`, and `pnpm build` from `frontend/` and resolve regressions introduced by the change.
- [ ] 5.2 Exercise the drawer at desktop and representative narrow/short mobile viewport sizes with safe-area emulation, mouse/touch input, and keyboard-only input; verify the sheet remains within the dynamic viewport, header and close action stay reachable, controls meet 44 by 44 CSS pixels, long text does not overflow, row swipes scroll without dragging, holding a dragged row at either visible list edge reaches initially hidden positions, focus is trapped/restored, and failure feedback is readable.