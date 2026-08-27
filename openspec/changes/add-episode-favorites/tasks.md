# Add Episode Favorites — Tasks

## 1. Database migration

- [x] 1.1 Create `migrations/20260827000001_add_episode_favorite.up.sql` adding `favorite BOOLEAN NOT NULL DEFAULT FALSE` to `episodes`
- [x] 1.2 Create `migrations/20260827000001_add_episode_favorite.down.sql` dropping the `favorite` column
- [x] 1.3 Run `sqlx migrate run` (or the project's migration path) and confirm the column exists with `favorite=false` on existing rows

## 2. Backend model

- [x] 2.1 Add `pub favorite: bool` to `Episode` in `src/models/episode.rs`, populated in `from_row` (default false for rows read before the field)
- [x] 2.2 Add `favorite` to the full-row `UPDATE` in `Episode::update` (single-row, no self-join) and to any test fixtures constructing `Episode`
- [x] 2.3 Add `Episode::set_favorite_by_yt_id(pool, yt_id, favorite) -> Result<Episode, Error>` resolving the row by `yt_id` and running a targeted `UPDATE episodes SET favorite = $2, updated_at = $3 WHERE id = $1 RETURNING *`, returning 404-equivalent error when no row matches

## 3. Backend API

- [x] 3.1 Add `#[put("/episodes/{yt_id}/favorite/")]` handler in `src/handlers/episodes.rs` with `FavoriteBody { favorite: bool }`, responding 204 on success and `CResponse::ko` with the failure status otherwise
- [x] 3.2 Register the handler in `src/handlers/mod.rs` next to the other episode services (under `RequireSession`)
- [x] 3.3 Verify `favorite` serializes in episode payloads (all-episodes and channel episodes endpoints) — extend a backend test or API smoke check

## 4. Pruning exempts favorites

- [x] 4.1 Modify `clean_channel` in `src/utils/worker.rs`: iterate newest-first, skip favorite episodes entirely, and delete a non-favorite only when the running count of non-favorites exceeds `max`
- [x] 4.2 Confirm `clean_orphan_files` and the sync window (`select_window`) are untouched by this change
- [x] 4.3 Add/extend unit tests for the prune semantics: favorites never deleted, 5 non-fav + 1 fav with `max` 5 deletes nothing, oldest favorite survives new arrivals, oldest non-favorite evicted when non-favorites exceed `max`

## 5. Frontend types and API client

- [x] 5.1 Add `favorite: boolean` to `Episode` in `frontend/src/types.ts`
- [x] 5.2 Add `setEpisodeFavorite(yt_id: string, favorite: boolean)` in `frontend/src/lib/api/client.ts` calling `PUT /api/1.0/episodes/{yt_id}/favorite/`

## 6. Favorites store

- [x] 6.1 Create `frontend/src/stores/favorites.ts` with `useFavoritesStore`: `byId` map + `favoriteIdSet` computed, `sync(episode)` merging loaded flags, `set(id, favorite)` calling the API and updating local state on success
- [x] 6.2 Add a frontend test for store sync/toggle behavior if the project's store tests cover playlist equivalents

## 7. Episode card favorite toggle

- [x] 7.1 Add a star toggle to `EpisodeCard.vue`: hollow (`PhStar`) when not favorite, filled (`PhStarFill`) when favorite, calling `favorites.set` and showing success/failure notifications (mirror the playlist toggle)
- [x] 7.2 Call `favorites.sync(episode)` when the card's episode changes so state stays in sync across views
- [x] 7.3 Extend/extend component tests (`EpisodeCard.test.ts`) covering mark/unmark and state sync

## 8. View filters and i18n

- [x] 8.1 Add a small favorites predicate helper next to `filterBySearchWords` in `frontend/src/lib/utils/list.filter.ts`
- [x] 8.2 Add a favorites-only toggle to `EpisodesView.vue` filtering `filteredEpisodes` by the store's per-id flag (combined with the search query), with an empty state when the filter matches nothing
- [x] 8.3 Add the same favorites-only toggle to `HistoryView.vue` using the shared helper, combined with its live search filter
- [x] 8.4 Add i18n keys (toggle success/failure, filter label, empty state) to all locales under `frontend/src/i18n/locales/` following the `playlist.*` style, reused by both views

## 9. Verification

- [x] 9.1 Run backend tests (`cargo test`) including the new prune tests
- [x] 9.2 Run frontend tests and build (`npm test`, `npm run build` in `frontend/`)
- [ ] 9.3 Manual end-to-end check: mark a favorite, publish/refresh past `max`, confirm the favorite survives and non-favorite eviction still works
- [ ] 9.4 Manual UI check: favorites-only filter in the episodes view and the history screen hides non-favorites, combines with search, and restores the full list when deactivated