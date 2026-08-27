# Add Episode Favorites — Design

## Context

u2vpodcast syncs YouTube channels into per-channel audio episode stores. Each channel has a retention limit `max`: `clean_channel` (in `src/utils/worker.rs`) reads the channel's episodes newest-first and deletes the files and rows of every episode beyond `max`, so a channel configured for 5 episodes keeps only the 5 newest.

Today there is no way to pin an episode: any episode can be evicted at the next sync once newer episodes arrive, including one the user wants to keep. The playlist feature (recently shipped) established the project's pattern for a per-episode toggle: a boolean-ish membership exposed by card buttons, backed by small REST endpoints and a Pinia store that drives button state.

This change adds a `favorite` flag on episodes: favorited episodes are never evicted and do not count toward `max`, exactly as the user described — e.g. with `max` 5, 5 non-favorites + 1 favorite means nothing is evicted when a 6th episode arrives; only if non-favorites exceed 5 does the oldest non-favorite go, never a favorite.

## Goals / Non-Goals

**Goals:**
- Persist a per-episode `favorite` flag in SQLite, defaulting to false for existing rows.
- Expose a protected endpoint to set/unset the flag, mirroring the existing progress endpoints (`PUT /api/1.0/episodes/{yt_id}/progress/`).
- Return `favorite` in every episode payload so UIs render state without extra lookups.
- Change `clean_channel` pruning so favorites are never deleted and don't count toward `max`.
- Add a star-shaped favorite toggle to `EpisodeCard` (hollow when not favorite, filled when favorite) and a favorites-only filter to the episodes view and history screen, with i18n strings in all locales.

**Non-Goals:**
- Favorites do NOT change the download/sync window (`select_window`, `wanted = max + MARGIN`): we still fetch new episodes per the channel's limit; favorites only affect retention/pruning.
- No dedicated "favorites screen" or favorites feed; the existing episodes view gains a filter instead.
- No multi-user semantics: the app is single-admin per instance, so `favorite` is a plain column, not a per-user join table.
- No changes to channel config or the `max` validation (`channel-retention-limit` validation stays as-is).

## Decisions

### 1. Column on `episodes` rather than a join table

A single `favorite BOOLEAN NOT NULL DEFAULT FALSE` column on `episodes` (migration `20260827000001_add_episode_favorite`) is simplest: the app has exactly one administer and pruning is a single table scan. A join table would add nothing here. The down migration drops the column (`ALTER TABLE episodes DROP COLUMN favorite;`).

### 2. Pruning counts only non-favorites and never deletes favorites

`clean_channel` currently iterates `read_episodes_for_channel` (newest-first by `published_at`) and deletes any episode whose position index `>= max`. New logic keeps the same iteration order but:

```rust
let mut kept = 0usize;
for episode in episodes {
    if episode.favorite {
        continue;          // never counted, never deleted
    }
    kept += 1;
    if kept > max {        // oldest non-favorite beyond the limit
        // delete file + row (unchanged removal code)
    }
}
```

Because the list is newest-first, incrementing `kept` only for non-favorites enumerates non-favorites newest→oldest; the first non-favorites to pass `kept > max` are exactly the oldest non-favorites. Favorites are skipped entirely, so they can never be evicted, and when non-favorites ≤ `max` the loop deletes nothing even if total rows (with favorites) exceed `max`. This satisfies every scenario in the `channel-retention-limit` delta, including the user's two cases: "5 non-fav + 1 fav, 6th arrives → nothing deleted" and "oldest episode is a favorite → never evicted".

`clean_orphan_files` is untouched (it only removes files with no episode row).

### 3. API mirrors the progress endpoints

A `#[put("/episodes/{yt_id}/favorite/")]` handler registered next to the other episode services in `src/handlers/mod.rs`, body `{"favorite": bool}` (serde struct `FavoriteBody` in `src/handlers/episodes.rs`), responding 204 on success and `CResponse::ko` with the right status (404 when the `yt_id` resolves to no row) on failure. Model method `Episode::set_favorite_by_yt_id(pool, yt_id, favorite)` resolves the id (same pattern as `update_progress_by_yt_id`) then runs a targeted `UPDATE episodes SET favorite = $2, updated_at = $3 WHERE id = $1 RETURNING *`.

### 4. Payloads carry `favorite`

`Episode` struct gains `pub favorite: bool`, populated in `from_row`, preserving `#[serde(default)]`-style defaults for desc fields only. `Episode::update`'s full-row `UPDATE` gains `favorite` in the column list (keeping the no-self-join single-row semantics from `episode-persistence`). Because every episode endpoint serializes the `Episode` struct, `favorite` flows into `/episodes/`, `/channels/{channel}/episodes/`, and the playlist/all-episodes payloads automatically.

### 5. Frontend: Pinia favorites store driving card state

Follows the `usePlaylistStore` precedent exactly:

- `frontend/src/stores/favorites.ts`: `useFavoritesStore` holding `byId = ref<Map<number, boolean>>` plus `favoriteIdSet` computed; `sync(episode)` merges a loaded episode's flag, `set(id, favorite)` updates locally after the API call succeeds (optimistic flip + rollback is a detail for implementation; playlist store reloads instead).
- `EpisodeCard.vue`: a star button (`PhStar` hollow / `PhStarFill` filled) toggling via `favorites.set(id, !favorite)`, showing success/failure notifications like the playlist toggle. Cards call `sync(episode)` when props change so a card in the channel view and one in the episodes view agree (the `episode-cards` "in sync everywhere" scenario).
- `EpisodesView.vue` and `HistoryView.vue`: a favorites-only toggle that combines with the existing search filter over the loaded list using the store's per-id flag; empty-state message when the filter is on and nothing matches. Both views use the same tiny predicate helper (beside `filterBySearchWords` in `frontend/src/lib/utils/list.filter.ts`) so the filter behaves identically on the two screens.
- `frontend/src/types.ts`: `favorite: boolean` on `Episode`; `frontend/src/lib/api/client.ts`: `setEpisodeFavorite(yt_id, favorite)` → `PUT`.

### 6. i18n

New keys (toggle success/failure, filter label, empty state) added to every locale under `frontend/src/i18n/locales/`, following the existing `playlist.*` key style.

## Risks / Trade-offs

- **Full-row `Episode::update` now writes `favorite`**: the download worker constructs episodes with `favorite: false`; a raced favorite mark could theoretically be overwritten by the worker, but the worker only creates new rows or updates metadata for rows it just fetched, so a mark landing between fetch and update for the same row is the same race the existing row update already has for `listen`/progress. Mitigated by making `set_favorite` a targeted `UPDATE` that touches only the flag.
- **Pruning semantics change is subtle**: the newest-first + non-favorite counter relies on `published_at DESC` order. `read_episodes_for_channel` already guarantees that order and has tests; the pruning change keeps the same source, and unit tests for `clean_channel`-level behavior (view logic) plus the existing `select_window` tests guard regressions. A future tie in `published_at` does not matter for pruning correctness (any of the tied oldest non-favorites is a valid eviction candidate).
- **Store staleness**: favorite state shown by cards depends on loaded episode payloads. Any view that loads fresh episodes updates the store via `sync`, so a stale card is only possible for an episode that never appears in any loaded list — acceptable and self-healing on next load.
- **DB migration cost**: `ALTER TABLE ... ADD COLUMN ... DEFAULT FALSE` is metadata-only in SQLite — no table rewrite, negligible for any episode count.