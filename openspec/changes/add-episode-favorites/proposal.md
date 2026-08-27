# Add Episode Favorites

## Why

Users want to keep specific episodes indefinitely, but the per-channel retention limit (`max`) deletes the oldest episodes when a channel grows beyond its limit. There is currently no way to protect an episode from pruning, so a long-form or evergreen episode is silently removed when a newer one is published.

## What Changes

- Add a persisted `favorite` flag on episodes; users can mark/unmark any episode as favorite from its card.
- Channel pruning (`clean_channel`) never deletes a favorited episode and does not count favorites toward the channel's `max` limit. Concretely: pruning only evicts non-favorite episodes, and only when the number of **non-favorite** episodes exceeds `max`.
- The episodes API returns the favorite flag; a dedicated endpoint toggles/sets it.
- The episode card gains a favorite toggle shown as a star icon (hollow when not favorite, filled when favorite) with a favorites-only filter in the episodes view and the history screen.

## Capabilities

### New Capabilities
- `episode-favorites`: Persisted favorite mark on episodes, a set/toggle API endpoint, the favorite flag in episode payloads, and the card/UI toggle plus a favorites-only filter.

### Modified Capabilities
- `channel-retention-limit`: Pruning requirements change to exempt favorited episodes from deletion and from the count used to decide eviction.
- `episode-persistence`: The episodes table stores a `favorite` flag; reads and updates preserve it.
- `episode-cards`: Episode cards render a favorite toggle and reflect the favorite state.
- `all-episodes-api`: Episode payloads include the favorite flag; a new endpoint exposes setting it.

## Impact

- **Database**: new `favorite` column on `episodes` (new migration, e.g. `20260827000001_add_episode_favorite`), defaulting to `false` for existing rows.
- **Backend (Rust, actix-web)**: `src/models/episode.rs` (column, read/update, new favorite methods), `src/utils/worker.rs` (`clean_channel` pruning logic), `src/handlers/episodes.rs` (new favorite endpoint + registration in `src/main.rs` or the handler router).
- **Frontend (SvelteKit SPA)**: `frontend/src/types.ts` (Episode model), `frontend/src/lib/api/client.ts` (favorite API calls), `frontend/src/components/EpisodeCard.vue` (star toggle), `frontend/src/views/EpisodesView.vue` and `frontend/src/views/HistoryView.vue` (favorites-only filter), i18n keys.
- **Tests**: pruning-unit tests in `worker.rs` (`clean_channel` semantics) and frontend component tests.
- No changes to channel config, auth, or the download/yt-dlp pipeline.