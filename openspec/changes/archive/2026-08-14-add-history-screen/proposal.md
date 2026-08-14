## Why

Today episodes are only reachable through a single channel's page, so users must remember which channel an episode belongs to before they can find it. As the number of channels grows, there is no single place to see everything in chronological order. We need a history screen that shows every episode from every channel, newest first, with the owning channel clearly identified on each card.

## What Changes

- Add a new `history` screen to the Vue SPA listing every episode from every channel, ordered by `published_at` descending (newest first).
- Reuse the existing episode card (with its built-in player) on the history screen, but render it in a more compact vertical form and identify the owning channel by name on each card.
- Add a word-based search field at the top of the history screen, matching the behavior of the channel episodes list (case-insensitive, matches every whitespace-separated word against `title`, `description`, and `yt_id`).
- Add a new backend endpoint that returns all episodes across channels, each annotated with its channel `slug` and `title`, sorted newest first.
- Add a navigation entry point to reach the history screen.

## Capabilities

### New Capabilities
- `all-episodes-api`: a backend API endpoint that returns all episodes across all channels, newest first, each annotated with its owning channel's `slug` and `title`.
- `history-screen`: a new Vue SPA screen that renders the cross-channel episode history in a compact, wider card layout with per-card channel identification and word-based search filtering.

### Modified Capabilities
- None.

## Impact

- **Code**:
  - Backend (`src/models/episode.rs`, `src/handlers/episodes.rs`, `src/handlers/mod.rs`): add a `channel_title` field to the episode payload and a handler/route for reading all episodes across channels with channel slug/title joined.
  - Frontend: new `frontend/src/views/HistoryView.vue`; router entry in `frontend/src/router/index.ts`; API client method in `frontend/src/lib/api/client.ts`; type updates in `frontend/src/types.ts`; a compact variant of `frontend/src/components/EpisodeCard.vue` (or a new compact card component) that shows the channel name.
- **APIs**: new `GET /api/1.0/episodes/` endpoint (protected by the existing session middleware); no changes to existing endpoints.
- **Dependencies**: none (reuses existing Vue Router, Pinia stores, `SearchInput`, and `filterBySearchWords`).
- **DB**: none (read-only over existing `episodes` + `channels` tables).
- **Frontend**: new route, new view, updated card component, updated types.
