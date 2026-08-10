## Why

As the number of channels and episodes grows, the channels page and the per-channel episodes page become hard to navigate: users must scroll through every card to find a specific channel or episode. There is currently no way to filter the lists by name, so finding content requires knowing exactly where it is.

## What Changes

- Add a text search input above the channels list on the homepage that filters the visible channel cards as the user types, matching against the channel's `title`, `description`, `url`, and `slug` (case-insensitive substring match).
- Add a text search input above the episodes list on each channel's episode page that filters the visible episode cards as the user types, matching against the episode's `title`, `description`, and `yt_id` (case-insensitive substring match).
- Filtering is client-side and live: the lists update on every keystroke with no server request, reload, or navigation.
- A channel/episode is shown only when every whitespace-separated word in the query is found somewhere in the matched fields (word-based filtering), so typing `linux tapas` matches items containing both words in any order.
- Clearing the search input restores the full list.
- Empty search results render a small "no matches" message instead of an empty page.

## Capabilities

### New Capabilities
- `list-search`: client-side, live word-based search filtering over the channel and episode lists in the SvelteKit frontend.

### Modified Capabilities
- None.

## Impact

- **Code**: `frontend/src/routes/+page.svelte` (add search input, derive filtered channels, apply filter before pagination), `frontend/src/routes/[id]/+page.svelte` (add search input, derive filtered episodes). Optionally a small shared helper for the word-based matching logic in `frontend/src/lib/utils/`.
- **APIs**: none. Filtering is purely client-side; no backend endpoint changes.
- **Dependencies**: none (SvelteKit reactivity and existing `flowbite-svelte` components are sufficient).
- **DB**: none.
- **Frontend**: the two list pages gain a search input and filtered rendering.
