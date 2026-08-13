## Context

The SvelteKit SPA renders two lists:

- **Homepage** (`frontend/src/routes/+page.svelte`): a list of `Channel` cards, with client-side pagination. `channels` is loaded in `+page.ts`, sliced by `getPaginatedChannels(currentPage, channels)` into `paginatedChannels`, and rendered in a `{#each}`. Pagination is driven by a `page` URL query param.
- **Channel page** (`frontend/src/routes/[id]/+page.svelte`): a list of `Episode` cards rendered directly from `data.episodes` in a `{#each}`. No pagination.

Both pages already receive the full list from their `load` functions, so filtering is purely a rendering concern — no new data fetching. Styling uses Tailwind + `flowbite-svelte` components. See `proposal.md - Why` for motivation and `specs/list-search/spec.md` for the required behavior.

## Goals / Non-Goals

**Goals:**
- Add a search input above the channel list and above the episode list.
- Filter visible cards live, per keystroke, with a shared word-based, case-insensitive matcher.
- Filter channels *before* pagination, so pagination continues to work on the filtered set.
- Show a "no matches" message when a non-empty query yields nothing.

**Non-Goals:**
- Server-side search or new API endpoints.
- Debouncing, highlighting matches, or fuzzy matching.
- Filtering across other fields (e.g., date, duration, listen status).
- Persisting the query in the URL or store.
- Changing any backend code.

## Decisions

### Decision 1: Shared pure matcher function in `frontend/src/lib/utils/helpers/`

Add a small pure helper (e.g. `helpers/list.filter.ts`) exposing `filterBySearchWords<T>(items: T[], query: string, getHaystack: (item: T) => string): T[]`.

- `query.trim().split(/\s+/)` yields the words; an empty/blank query returns all items unchanged.
- An item matches when every word appears (case-insensitively, via `toLowerCase()`) in `getHaystack(item).toLowerCase()`.
- `getHaystack` joins the fields per list: channels → `[title, description, url, slug].join(' ')`, episodes → `[title, description, yt_id].join(' ')`.

**Why**: keeps the matching logic in one testable place instead of duplicating it in both pages. The project already keeps pure helpers under `frontend/src/lib/utils/helpers/` (e.g. `input.validation.ts`), including unit tests next to them.

**Alternative considered**: inline per-page `$:` reactive filters. Rejected: duplicates the word-splitting and matching logic twice and makes a future third list (or per-page test) copy it again.

### Decision 2: Reusable `SearchInput.svelte` component

Create `frontend/src/lib/components/SearchInput.svelte` with a `bind:value`-able bound variable (`export let value: string`) and optional `placeholder` prop, styled with existing Tailwind/flowbite patterns.

**Why**: both pages need an identical input above a card grid; one component avoids two copies of the same markup and CSS.

**Alternative considered**: inline `<input>` in each page. Rejected for the same DRY reason as Decision 1.

### Decision 3: Filter channels before pagination

On the homepage, derive the filtered list first, then paginate the filtered result:

```
$: filteredChannels = filterBySearchWords(channels, searchQuery, haystackOfChannel);
$: currentPage = getCurrentPage(...);
$: paginatedChannels = getPaginatedChannels(currentPage, filteredChannels);
```

**Why**: pagination (`page` query param) currently operates on the whole `channels` array; if filtering ran after pagination, items could be hidden by a query yet still occupy a page slot, producing empty pages. Filtering first keeps pagination semantics consistent with the visible list. The existing `pageUrl`/`goToPage` logic and the `page` param behavior are otherwise untouched. No change to pagination behavior itself.

### Decision 4: Episodes page filters the whole list, no pagination changes

On the channel page, add `searchQuery` state and render `filteredEpisodes = filterBySearchWords(data.episodes, searchQuery, haystackOfEpisode)` in the existing `{#each}`. Because there is no pagination, the whole filtered array renders as-is.

**Why**: minimal change to an already-simple page; the `{#each}` just iterates the derived array.

### Decision 5: Empty state message

When the trimmed query is non-empty and the filtered array is empty, render a message (e.g. "No results match your search.") in place of the `{#each}` block on the affected page.

**Why**: the spec requires a visible no-matches message rather than a blank page.

## Risks / Trade-offs

- **[Risk] Very long lists re-filter on every keystroke.** The matcher is O(n) per keystroke over the full list. → Mitigation: lists here are small (a handful of channels; episodes per channel are bounded by the download backlog). No debounce needed for expected data sizes; if lists grow later, a debounce can be added without spec changes.
- **[Trade-off] Lowercasing both sides per item per keystroke** re-computes `toLowerCase()` repeatedly. → For expected list sizes this is negligible; a precomputed normalized haystack would add complexity for no measurable benefit.
- **[Risk] Pagination interplay.** If a user filters on page 3 and the filtered result has fewer pages, `getPaginatedChannels` already clamps `start` to `0` when `start >= total`, so the filter never shows an empty page. → No code change needed, but verify in the task.

## Migration Plan

1. Add `filterBySearchWords` helper + unit test under `frontend/src/lib/utils/helpers/`.
2. Add `SearchInput.svelte` component.
3. Wire search into the homepage (filter → paginate → render) and the channel page (filter → render).
4. `npm run build` (or the project's frontend build) and `cargo build` to confirm nothing breaks; redeploy.
5. Manually verify per `tasks.md` scenarios.

**Rollback**: revert the frontend changes; no DB, API, or config changes exist to revert.

## Open Questions

None.
