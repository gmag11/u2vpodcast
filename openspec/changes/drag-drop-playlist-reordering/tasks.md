## 1. Drag-and-Drop Foundation

- [x] 1.1 Add `vue-draggable-plus` to the frontend dependencies and verify `pnpm install --frozen-lockfile` succeeds with the updated lockfile.
- [x] 1.2 Add matching English and Spanish strings for the drag handle, pickup/move/drop/cancel announcements, pending state, and reorder failure; verify both locale files expose the same playlist keys and remove obsolete move-up/move-down strings if no longer used.

## 2. Playlist Store Persistence

- [x] 2.1 Update the playlist store reorder action to apply a valid complete ID order optimistically, serialize reorder requests, and retain a snapshot for rollback; verify store tests cover the immediate optimistic order and successful persistence payload.
- [x] 2.2 Restore or reload the latest confirmed playlist after reorder failure without overwriting a concurrent removal, and verify store tests cover ordinary rollback plus an item removed while the request is pending.

## 3. Inline Playlist Reordering

- [x] 3.1 Replace each playlist row's stacked caret buttons with one six-dot handle immediately to the left of the existing `EpisodeCard`, make the complete wrapper the sortable item, and remove the old `move()` path; verify a view test asserts one handle per row, no caret controls, and unchanged card actions.
- [x] 3.2 Bind the playlist rows to `vue-draggable-plus` through the handle only, style the active row and insertion position, commit the full changed ID order, and skip unchanged drops; verify view tests cover the resulting payload, no-op drops, and disabled handles while persistence is pending.
- [x] 3.3 Configure touch delay/tolerance, a minimum 44 by 44 CSS-pixel handle target, and handle-only drag behavior so tapping or swiping elsewhere preserves card actions and page scrolling without horizontal overflow; verify focused mobile interaction tests cover drag activation and ordinary card scrolling.
- [x] 3.4 Configure viewport edge auto-scroll so a drag can reach cards above or below the visible page; verify tests cover both directions, stopping outside the edge zone and at document boundaries, and preserving the active drag until an off-screen position is reached.
- [x] 3.5 Implement keyboard pickup, arrow movement, drop, and cancellation on the same order path, retaining handle focus and announcing state through a live region; verify tests exercise the complete keyboard sequence and announcements.
- [x] 3.6 Reconcile rendered rows with external store mutations, reject stale-ID drops, and show a localized failure notification after rollback; verify tests cover completion-driven removal during a drag and reorder failure.

## 4. Playback Integration

- [x] 4.1 Confirm play-all, episode playback, playlist removal, and completion-driven removal still use the store's current order after inline reordering; verify the focused playlist view and player tests pass.

## 5. Verification

- [x] 5.1 Run `pnpm test`, `pnpm typecheck`, `pnpm lint`, and `pnpm build` from `frontend/` and resolve regressions introduced by the change.
- [x] 5.2 Exercise the main playlist at desktop and representative narrow/short mobile viewport sizes with mouse, touch emulation, and keyboard-only input; verify the grip stays visually compact, its touch target is at least 44 by 44 CSS pixels, cards do not overflow, card controls remain operable, insertion feedback is visible, and holding a dragged row at either viewport edge reaches initially hidden cards.