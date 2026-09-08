## 1. Sorting utility

- [x] 1.1 Add `frontend/src/lib/utils/channel.sort.ts` exporting `ChannelSortKey` (`'last_date' | 'title' | 'id'`), `SortDirection` (`'asc' | 'desc'`), and `sortChannels(channels, key, direction)` returning a new sorted array without mutating the input
- [x] 1.2 Implement per-key comparators: `last_date` (nulls always last), `title` (case-insensitive), `id` (numeric); direction flips the ordering and falls back to `last_date` / `desc` defaults on unknown values
- [x] 1.3 Add `frontend/src/lib/utils/channel.sort.test.ts` covering each key × direction, `last_date` null handling, and input immutability
- [x] 1.4 Run `pnpm test` to confirm the utility tests pass

## 2. Channels view integration

- [x] 2.1 In `frontend/src/views/ChannelsView.vue`, replace the hardcoded `sortedChannels` computed with `sortChannels(channels, sortKey, sortDirection)`, driven by refs defaulting to `'last_date'` / `'desc'`
- [x] 2.2 Add `localStorage` persistence for the selection (resolve initial value defensively on mount, write on change) following the `theme.ts` pattern
- [x] 2.3 Add sort controls (key picker + direction toggle) next to `SearchInput` in the toolbar row, reflecting the current selection

## 3. Verify

- [x] 3.1 Run `pnpm test`, `pnpm run lint`, and `pnpm run build` in `frontend/`
- [x] 3.2 Manually confirm the three keys and both directions re-order the list and persist across reload
