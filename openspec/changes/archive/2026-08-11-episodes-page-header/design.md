## Context

`EpisodesView.vue` (route `/:channelId(\d+)`) lists episodes for a numeric channel id. It already resolves `channelSlug` from the first episode's `channel_slug` (used by the Refresh button) and has a `resolveSlugFallback()` that calls `GET /api/1.0/channels/` when the list is empty. The episodes API does not return the channel title, but the channels endpoint does (`Channel.title`).

## Goals / Non-Goals

**Goals:**
- Show the current channel title at the top of the episodes page.
- Provide a left arrow that navigates to `/` (channel list).
- Reuse existing API/design tokens; no backend change.

**Non-Goals:**
- No backend/API changes (title comes from the existing channels endpoint).
- No change to the shared app header (the new header is page content).
- No breadcrumbs or multi-level navigation.
- No cover image in the header (title + arrow only).

## Decisions

### D1: Resolve the channel title during page load

In `load()`, after fetching episodes, also fetch the channel list (`api.getChannels()`) and find the channel whose `id === Number(route.params.channelId)`. Store `channelTitle` in a ref. When no channel matches, set a fallback string ("Episodes"). The slug fallback logic already fetches channels when needed; consolidate so the title and slug resolution share one channel-list lookup.

**Alternatives considered:** Extending the episodes API to return `channel_title` — rejected, backend change not needed; relying on `episodes[0].channel_slug` — rejected, it gives a slug not a title and fails on empty lists.

### D2: Header markup below the app header

Place the header as the first element inside the page `<main>` (or a wrapper div before the search bar), with left arrow + title. Left arrow uses a Phosphor icon (e.g., `PhArrowLeft`) as a button that calls `router.push({ name: 'channels' })` (route `/`). Styling: `font-display`, title sizing consistent with the Channels dashboard "Dashboard" heading; arrow as a bordered/ghost icon button for touch target.

### D3: No new route

The back arrow targets the existing `channels` route by name — no router changes needed.

## Risks / Trade-offs

- **Extra channels fetch per episodes page load** → One lightweight `GET /api/1.0/channels/` request; acceptable, and it also feeds the Refresh-button slug fallback.
- **Title mismatch if channel renamed mid-session** → Title is read fresh on load; refresh button also re-reads channels, so it self-corrects.
- **Fallback when empty** → "Episodes" keeps the header usable if the channel lookup fails.
