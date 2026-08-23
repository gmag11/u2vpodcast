## Context

The app has a single admin user; there is no user management (the auth layer only gates access). Consequently server-persisted state is app-global — playlists do not need per-user scoping. The backend is Actix-web + sqlx/SQLite; the auth-required scope is `RequireSession` in `src/handlers/mod.rs`, responses use `CResponse::ok(session, data)`.

Current player mechanics (earlier steps): the store seeds an up-next queue from a context list; `ended` consumes the queue; long-press next marks the current episode listened (shared `markListened()` path, step 2/3); `listen` + `position_seconds`/`listened_at` persist server-side (step 3).

## Goals / Non-Goals

**Goals:**
- One server-persisted, explicitly ordered playlist (pending episodes) per instance.
- Add, remove, reorder, and playback seeding of the playlist.
- Completion (or long-press mark) removes the episode from the playlist and marks it listened.
- "Mark as not listened" control re-appends the episode to the end.

**Non-Goals:**
- Multiple named playlists, playlist CRUD/rename.
- Per-user ownership (`user_id`) while the app has a single user.
- Drag & drop reorder (up/down controls first; DnD is a later extension).

## Decisions

### Decision 1: Schema — single table, no user_id

```sql
CREATE TABLE IF NOT EXISTS playlist_items (
    id INTEGER PRIMARY KEY AUTOINCREMENT NOT NULL,
    episode_id INTEGER NOT NULL,
    position INTEGER NOT NULL,
    added_at DATETIME NOT NULL,
    UNIQUE(episode_id)
);
```

No `user_id` (single-user app); `UNIQUE(episode_id)` guarantees an episode appears at most once. Referential integrity to `episodes` is handler-enforced, consistent with the existing schema (no FK constraints on `episodes.channel_id` either). If multi-user ever lands, `user_id` is a later migration plus backfill.

**Why**: minimal schema fitting the actual deployment; the unique constraint is the natural playlist invariant.

### Decision 2: The playlist is a pending list; finishing removes the episode

Semantics:
- `ended` → `markListened()` (step 3: `listen=true`, `position_seconds=duration`) **and** remove the episode from the playlist (`DELETE /api/1.0/playlist/{episode_id}/`).
- Long-press next (step 2, `skipNext({markCurrent:true})`) — same removal, same mark path.
- Short-press next does not mark listened → the episode stays in the playlist.

The frontend tracks which plays came from the playlist (`queueSource: 'playlist' | 'list'` set when seeding). On a mark-listened event, if the current episode came from the playlist source, the store also calls the playlist removal. Removal is fire-and-forget; if it fails the mark is still saved and the mismatch is reconciled on the next playlist read.

**Why**: the user-defined model — "al finalizar el episodio se marca como reproducido y se elimina de la playlist". Keeping the removal client-driven avoids inventing a combined server endpoint that mixes progress and queue concerns.

### Decision 3: API surface (singular resource, under `RequireSession`)

| Method | Path | Behavior |
|---|---|---|
| GET | `/api/1.0/playlist/` | items in `position ASC`, joined with channels (slug/title) |
| POST | `/api/1.0/playlist/` | `{ episode_id }` → append at `max(position)+1`; conflict if already present |
| DELETE | `/api/1.0/playlist/{episode_id}/` | remove + reindex contiguous positions |
| PUT | `/api/1.0/playlist/reorder/` | `{ episode_ids: [...] }` → validate same set, rewrite positions in given order |

All responses use `CResponse` (session present for the existing envelope). No `from_session` ownership checks because the resource is global.

**Why**: mirrors `users.rs`/`channels.rs` handler style while dropping all playlist-name CRUD that no longer exists.

### Decision 4: Rust model `src/models/playlist.rs`

`PlaylistItem` struct with `from_row`/`from_row_with_channel` helpers; methods `read_all`, `add`, `remove` (reindex), `reorder`, `read_episodes_with_channels` (INNER JOIN episodes+channels ordered by position). Tests follow the memory-pool pattern.

**Why**: one cohesive model module per resource, matching `channel.rs`/`user.rs`.

### Decision 5: Frontend state and routes

- `frontend/src/stores/playlists.ts`: `items` (ordered episodes), `episodeIdSet` (for card toggles), actions `load()`, `add(episodeId)`, `remove(episodeId)`, `reorder(episodeIds)`.
- Single route `/playlist` in `frontend/src/router/index.ts` with header nav beside History.
- `PlaylistView.vue`: title "Playlist", ordered `EpisodeCard`s with up/down reorder, remove, play-all / per-card play (seeds queue with `queueSource='playlist'`), plus an empty state.

**Why**: with a single playlist there is no list-of-plays screen — one view suffices. The store exposes the id set so cards render the toggle state without refetching.

### Decision 6: Add/remove toggle and unmark on episode cards

`EpisodeCard` gains:
- an add/remove toggle for the playlist (icon flips between "add to playlist" and "remove from playlist" based on `episodeIdSet`), with a notification on action;
- the "mark as not listened" control when `props.episode.listen` is true: `updateEpisodeProgress(id, { position_seconds: 0, listened: false })` (step 3 endpoint) **and** `add(episodeId)` to re-append at the end of the playlist, then refresh card state.

The unmark depends on step 3's endpoint accepting `listened: false` (it does: `listen` is a plain boolean field). If the playlist add fails, the mark is still cleared.

**Why**: both actions live on the card, the shared episode surface across all views. The re-append implements "al ponerlo como no reproducido se añadirá al final de la playlist". Position resets to 0 so the re-added episode starts from the beginning (consistent with the resume policy in step 3).

### Decision 7: `queueSource` flag for completion removal

The player store seeds `queueSource` when a list is provided to `play()`: `'playlist'` when the list came from the playlist view, `'list'` otherwise. `advance()`/completion and long-press mark consult it to decide whether to also delete the finished episode from the playlist. The flag is per-queue-seed and re-seeded on every play.

**Why**: the store does not know the origin of the list otherwise; this is the minimal bridge between playback and playlist lifecycle.

## Risks / Trade-offs

- **[Risk] Removal races (ended + long-press near-simultaneous).** The mark path is idempotent and the removal is a delete-by-id; a second delete is a no-op (404 ignored).
- **[Risk] Playlist add fails transiently.** Notifications surface errors; next load reconciles.
- **[Trade-off] Single global playlist couples all users if multi-user arrives later.** Documented; migration adds `user_id` when roles land.
- **[Risk] Reorder with episodes removed server-side (episode deleted).** `read_episodes_with_channels` uses INNER JOIN; stale items disappear naturally and `reorder` only rewrites surviving ids.

## Migration Plan

1. Add migration for `playlist_items` (single table, no user_id).
2. Backend: `playlist.rs` model + `playlists.rs` handlers + route registration.
3. Frontend: API client, Pinia store, route + header, `PlaylistView`, card add/remove toggle, unmark flow.
4. i18n strings (en/es).
5. Tests: Rust model/handler (memory pool), Vitest store + card toggle; manual verification per `tasks.md`.

**Rollback**: revert code; `down.sql` drops the table.

## Open Questions

None.