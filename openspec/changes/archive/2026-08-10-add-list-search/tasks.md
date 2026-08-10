## 1. Shared search logic

- [x] 1.1 Create `frontend/src/lib/utils/helpers/list.filter.ts` exporting `filterBySearchWords<T>(items: T[], query: string, getHaystack: (item: T) => string): T[]`. The function splits the trimmed query on whitespace into words; when there are no words it returns `items` unchanged; otherwise it returns only items for which every word is contained case-insensitively (via `toLowerCase()`) in `getHaystack(item).toLowerCase()`.
- [x] 1.2 Add a unit test file `frontend/src/lib/utils/helpers/list.filter.test.ts` covering: empty/blank query returns all items, single-word case-insensitive match, multi-word AND matching in any order, and no-match returning an empty array.

## 2. Search input component

- [x] 2.1 Create `frontend/src/lib/components/SearchInput.svelte` with `export let value: string` bound via `bind:value`, an optional `placeholder` prop, and styling consistent with the existing Tailwind/flowbite-svelte UI. It must be a plain text input that updates `value` on every keystroke.

## 3. Homepage (channels list)

- [x] 3.1 In `frontend/src/routes/+page.svelte`, import `SearchInput` and `filterBySearchWords`, and add `let searchQuery = '';` state. Render `<SearchInput bind:value={searchQuery} />` above the channels grid.
- [x] 3.2 Derive `filteredChannels` with `filterBySearchWords(channels, searchQuery, (c) => [c.title, c.description, c.url, c.slug].join(' '))`, and change `getPaginatedChannels(currentPage, ...)` to receive `filteredChannels` so filtering happens before pagination. Verify pagination clamps correctly when the filtered set shrinks (existing `start >= total` handling).
- [x] 3.3 When `searchQuery.trim()` is non-empty and `filteredChannels.length === 0`, render a "no results match your search" message instead of the `{#each}` block.

## 4. Channel page (episodes list)

- [x] 4.1 In `frontend/src/routes/[id]/+page.svelte`, import `SearchInput` and `filterBySearchWords`, add `let searchQuery = '';`, and render `<SearchInput bind:value={searchQuery} />` above the episodes grid.
- [x] 4.2 Derive `filteredEpisodes` with `filterBySearchWords(data.episodes, searchQuery, (e) => [e.title, e.description, e.yt_id].join(' '))` and iterate `filteredEpisodes` in the existing `{#each}`.
- [x] 4.3 When `searchQuery.trim()` is non-empty and `filteredEpisodes.length === 0`, render a "no results match your search" message instead of the `{#each}` block.

## 5. Verification

- [x] 5.1 Run the frontend unit tests and confirm the new `list.filter` tests pass. Vitest is not declared in `package.json` (to keep `pnpm install --frozen-lockfile` valid), so tests run in the Docker image builder via `pnpm dlx vitest@1.6.1 run` in the `frontend_builder` stage, using the new minimal `frontend/vitest.config.ts` (SvelteKit plugin excluded) that only matches `src/**/*.test.ts`.
- [x] 5.2 Build the frontend (`npm run build` in `frontend/`) and confirm no type or Svelte compilation errors. Build passed; `svelte-check` reports 29 pre-existing errors, none in the files touched by this change.
- [x] 5.3 Log into the app and verify the channels page: typing a title word filters the cards live, multi-word queries require all words, matching is case-insensitive, clearing restores the list, and a query with no matches shows the no-results message.
- [x] 5.4 Open a channel's episodes page and repeat the same checks against episode titles, descriptions, and yt_id.
- [x] 5.5 Confirm channels pagination still works with an active filter (filtered set is paginated, not the full list) and that filtering on a later page does not show an empty page.
