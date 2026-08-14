## 1. Backend: all-episodes endpoint

- [ ] 1.1 Add a `channel_title: String` field to the `Episode` struct in `src/models/episode.rs`, annotated with `#[serde(default = "get_default_empty")]` like `channel_slug`, and set it to an empty string in `from_row`, `new`, and `save`/`create`/`update` paths so existing endpoints serialize it as `""`.
- [ ] 1.2 Add `Episode::read_all_with_channels(pool)` in `src/models/episode.rs` that runs `SELECT e.*, c.slug AS channel_slug, c.title AS channel_title FROM episodes e LEFT JOIN channels c ON c.id = e.channel_id ORDER BY e.published_at DESC`, mapping via a `from_row_with_channel` helper that populates `channel_slug` and `channel_title`.
- [ ] 1.3 Add a `read_all` handler `#[get("/episodes/")]` in `src/handlers/episodes.rs` that calls `Episode::read_all_with_channels` and returns `CResponse::ok(session, episodes)`, mirroring the existing episodes handler error handling.
- [ ] 1.4 Register the new handler in `src/handlers/mod.rs` inside the `RequireSession` scope (next to the existing `episodes::read_with_pagination` service).
- [ ] 1.5 Run `cargo build` and confirm the backend compiles with the struct and route changes.

## 2. Frontend: API client and types

- [ ] 2.1 Add `channel_title: string` to the `Episode` interface in `frontend/src/types.ts`.
- [ ] 2.2 Add `getAllEpisodes()` to `frontend/src/lib/api/client.ts` returning `request<Array<Episode>>('/api/1.0/episodes/')`.

## 3. Frontend: compact episode card

- [ ] 3.1 Add an optional `compact?: boolean` prop to `frontend/src/components/EpisodeCard.vue` (default `false`).
- [ ] 3.2 In compact mode, reduce vertical padding, shrink the thumbnail, drop the description line, and render `episode.channel_title` as a small channel label above the title; keep all player controls unchanged. Verify the non-compact card (channel episodes page) renders identically to before.

## 4. Frontend: history view and route

- [ ] 4.1 Create `frontend/src/views/HistoryView.vue` that loads `api.getAllEpisodes()` on mount, applies the same session-guard pattern as `EpisodesView.vue`, and shows an empty-state message when there are no episodes.
- [ ] 4.2 In `HistoryView.vue`, add `SearchInput` bound to `searchQuery` and derive `filteredEpisodes` with `filterBySearchWords(episodes, searchQuery, (e) => [e.title, e.description, e.yt_id].join(' '))`, plus a "no results" message matching `EpisodesView.vue`.
- [ ] 4.3 Render the episodes in a wider container with a responsive grid (single column on small screens, two columns on large) using `EpisodeCard` with `:compact="true"`.
- [ ] 4.4 Add the `{ path: '/history', name: 'history', component: () => import('@/views/HistoryView.vue') }` route to `frontend/src/router/index.ts`.
- [ ] 4.5 Add a "History" navigation link to `frontend/src/components/AppHeader.vue` pointing at the `history` route.

## 5. Verification

- [ ] 5.1 Run the frontend typecheck/build (e.g. `pnpm run build` in `frontend/`) and confirm no type or compilation errors.
- [ ] 5.2 Log in and open the history screen: confirm episodes from all channels appear newest first, each card shows its channel name, and compact cards play audio with seek/stop/speed/volume working.
- [ ] 5.3 Confirm the history search filters live per keystroke (case-insensitive, multi-word AND, `yt_id` match), clearing restores the full list, and a no-match query shows the no-results message.
- [ ] 5.4 Confirm the history route redirects to `/login` when unauthenticated and is reachable via the header link when authenticated.
- [ ] 5.5 Confirm the existing channel episodes page still renders and plays correctly after the `Episode`/`EpisodeCard` changes.
