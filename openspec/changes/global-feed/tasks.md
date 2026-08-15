## 1. Backend: global feed endpoint

- [x] 1.1 Add a `get_global_feed` handler in `src/handlers/feed.rs` that calls `Episode::read_all_with_channels(&data.pool)`, builds items with enclosure `{url}/media/{channel_slug}/{yt_id}.mp3` and title prefixed with `channel_title` (e.g. `Confesiones de Gasolinera: Episodio 10`), and skips episodes whose `channel_slug` is empty so no broken enclosure is emitted.
- [x] 1.2 Keep the rest of the item shape consistent with `build_feed` (pubDate RFC 2822, description beginning with the YouTube link, iTunes extension with duration/summary).
- [x] 1.3 Register `web::resource("/feed.xml").route(web::get().to(get_global_feed)).wrap(SessionOrBasicAuth)` inside `web_feed` in `src/handlers/feed.rs`.
- [x] 1.4 Run `cargo build` and confirm the backend compiles.

## 2. Frontend: history download link

- [x] 2.1 In `frontend/src/views/HistoryView.vue`, import `baseEndpoint` from `@/lib/api/client` and `PhRss` from `@phosphor-icons/vue`.
- [x] 2.2 Add an RSS icon link in the header row (next to the title) pointing at `${baseEndpoint}/feed.xml`, with a tooltip (e.g. "Get global RSS feed"), mirroring the `ChannelCard.vue` feed link.
- [x] 2.3 Run the frontend typecheck/build (`pnpm build` in `frontend/`) and confirm no errors.

## 3. Verification

- [x] 3.1 Confirm `cargo build` and `pnpm build` pass with the new handler and link.
- [ ] 3.2 Request `/feed.xml` with valid credentials: every channel's episodes appear newest first, each `<enclosure>` resolves to `{url}/media/{slug}/{yt_id}.mp3`, and titles carry the channel prefix.
- [ ] 3.3 Confirm `/feed.xml` is rejected without credentials (same protection as per-channel feeds).
- [ ] 3.4 Open the history screen and confirm the RSS link is visible, points at the global feed, and opens it.
- [ ] 3.5 Confirm per-channel feeds at `/channels/{slug}/feed.xml` are unchanged.
