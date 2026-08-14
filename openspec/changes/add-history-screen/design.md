## Context

The app is a Rust (actix-web) backend serving a Vue 3 SPA. Episodes are reachable only through `GET /api/1.0/channels/{channel}/episodes/` on a per-channel page (`/:channelId`). The episode payload (`src/models/episode.rs`) already carries a `channel_slug` (populated by the per-channel handler) but no channel title, and there is no endpoint that returns episodes across channels.

The Vue SPA renders episode lists in `EpisodesView.vue` using `EpisodeCard.vue` (which embeds the full player controls via the Pinia `player` store) and filters with the shared `filterBySearchWords` helper plus `SearchInput.vue`. The channel list lives in `ChannelsView.vue`. Navigation is centralized in `frontend/src/router/index.ts` and the top bar in `AppHeader.vue`.

## Goals / Non-Goals

**Goals:**
- Expose all episodes across channels, newest first, each annotated with `channel_slug` and `channel_title`.
- Add a protected `history` route rendering those episodes in a compact, wider card layout that shows the owning channel name.
- Add live, word-based search over the history list, reusing the existing filter helper.

**Non-Goals:**
- Server-side search or pagination on the history endpoint.
- Marking episodes as listened, or any write/DB change.
- Changing the existing channel episodes page or its card layout.
- Reordering, grouping, or collapsing by channel.

## Decisions

### Decision 1: New backend endpoint `GET /api/1.0/episodes/`

Add a handler in `src/handlers/episodes.rs` and register it under the existing `RequireSession` scope in `src/handlers/mod.rs`. Add a `channel_title` field to the `Episode` struct (with `#[serde(default)]`, like `channel_slug`) and a new model method that reads all episodes joined to their channel:

```sql
SELECT e.*, c.slug AS channel_slug, c.title AS channel_title
FROM episodes e
LEFT JOIN channels c ON c.id = e.channel_id
ORDER BY e.published_at DESC;
```

**Why**: the existing `Episode::read_all` selects from `episodes` only and leaves `channel_slug` empty; the media URL and channel label both need per-episode channel data. A single `LEFT JOIN` avoids the N+1 lookups a per-episode channel fetch would cause, and a `LEFT JOIN` (rather than `INNER`) guarantees episodes whose channel is missing still serialize with empty `channel_slug`/`channel_title` rather than being dropped.

**Alternative considered**: frontend joins via `getChannels()` + `getAllEpisodes()` on `channel_id`. Rejected: it shifts channel-title resolution to every client and still needs a cross-channel episodes endpoint; annotating server-side keeps the client a single fetch.

### Decision 2: Reuse `EpisodeCard.vue` with a `compact` prop

Extend `EpisodeCard.vue` with an optional `compact?: boolean` prop rather than creating a separate card component. When `compact` is true: reduce vertical padding, shrink the thumbnail, drop the description line, and render the channel name (from `episode.channel_title`) as a small label above the title. The player control wiring (play/pause/seek/stop/speed/volume) is unchanged.

**Why**: the card already owns all player logic via the `player` store; a new component would duplicate that wiring. A prop keeps one source of truth and lets the history list diverge only in presentation.

**Alternative considered**: separate `HistoryEpisodeCard.vue`. Rejected: duplicates player interaction logic and risks the two cards drifting.

### Decision 3: `history` route and `HistoryView.vue`

Add route `{ path: '/history', name: 'history', component: HistoryView }` to `router/index.ts`; it is protected automatically because it has no `public` meta. `HistoryView.vue` fetches `api.getAllEpisodes()` on mount, applies the auth-session guard pattern used by `EpisodesView.vue`, and renders cards inside a wider container using a responsive two-column grid on large screens (single column on small) so the list uses width instead of height.

**Why**: matches the "more compact vertically, wider" requirement; a two-column grid shortens the scroll while each card remains compact.

### Decision 4: Channel name sourced from `channel_title` on the episode

Add `channel_title: string` to the `Episode` interface in `frontend/src/types.ts` and have `EpisodeCard` render it in compact mode. The `channel_slug` on each episode already enables the media URL without extra lookup.

**Why**: the backend is the single source of truth for the channel name; the client needs no channel list on the history screen.

### Decision 5: Search reuses the existing filter helper

`HistoryView.vue` uses `SearchInput` bound to `searchQuery` and derives `filteredEpisodes = filterBySearchWords(episodes, searchQuery, (e) => [e.title, e.description, e.yt_id].join(' '))`, matching `EpisodesView.vue`, with the same "no results" empty state.

**Why**: identical, already-tested behavior; no new filtering logic or component.

### Decision 6: History navigation entry

Add a "History" `router-link` to `AppHeader.vue` so the screen is reachable globally (shown only on authenticated routes, consistent with the rest of the header).

**Why**: a global header entry is the minimal, discoverable navigation point without adding a new nav system.

## Risks / Trade-offs

- **[Risk] Unbounded list size.** The endpoint returns every episode; with many channels this list grows and re-filters on each keystroke. → Mitigation: expected data sizes are modest (bounded by the download backlog); the filter is O(n) and lists are client-side only. If it grows, add pagination or server-side search later without spec change.
- **[Risk] `LEFT JOIN` serialization regression.** Adding `channel_title` to `Episode` changes the serialized shape of the existing per-channel episodes endpoint (which currently leaves `channel_slug` empty and would leave `channel_title` empty). → Mitigation: the new field defaults to `""`, so existing consumers are unaffected; verify the per-channel handler still works after the struct change.
- **[Trade-off] Compact card hides the description.** → Mitigation: description remains available on the channel episodes page; history prioritizes density per the requirement.
- **[Risk] Header nav change affects all pages.** → Mitigation: the History link is additive and uses existing header styles; no existing behavior is removed.

## Migration Plan

1. Backend: add `channel_title` to `Episode`, add `read_all_with_channels` + handler, register route. `cargo build` to confirm.
2. Frontend: add `channel_title` to `types.ts`, `getAllEpisodes()` to the API client, `HistoryView.vue`, `compact` prop on `EpisodeCard.vue`, route + header link.
3. Build the frontend (`pnpm build` / existing builder) and verify types.
4. Manually verify: history route lists all episodes newest-first with channel names, compact cards play audio, search filters live, no-matches message renders.

**Rollback**: revert the frontend and backend changes; no DB migration, config, or dependency changes exist.

## Open Questions

None.
