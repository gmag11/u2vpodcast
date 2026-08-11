## Why

The episodes page (`/app/{channelId}`) has no header identifying which channel is being viewed, and no way to navigate back to the channel list except the browser's back button. Users lose context when deep-linking into a channel and must manually navigate home.

## What Changes

- Add a page header to the episodes screen showing the current channel's title, preceded by a left arrow that navigates back to the channel list (`/`).
- Resolve the channel title for the episodes page: from the channel list API (`GET /api/1.0/channels/`) by matching the route's numeric channel id, since the episodes API does not return the channel title.

## Capabilities

### New Capabilities
- `episodes-page-header`: The episodes screen gains a header bar with the channel title and a back arrow to the channel list.

### Modified Capabilities
- `vue3-spa`: The episodes route (`/:channelId`) now resolves and displays the channel title in a header and provides a back navigation control.

## Impact

- **Frontend only.** `frontend/src/views/EpisodesView.vue` — add a header (back arrow + channel title) below the app header and above the search bar, and resolve the channel title during load.
- No backend/API changes; the title comes from the existing channels endpoint.
- No design-system changes; the header reuses existing tokens and Phosphor icons (left arrow).
