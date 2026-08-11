## 1. Backend — single-channel refresh

- [x] 1.1 Add `pub async fn update_channel(pool: &SqlitePool, channel_id: i64) -> Result<(), Error>` in `src/utils/worker.rs` that reads the channel by id, builds `Ytdlp` from `ytdlp_path()`/`cookies_file()` and the audios folder from `audios_dir()`, and runs the existing `process_channel` + `clean_channel` for that channel
- [x] 1.2 Refactor `do_the_work` to reuse `update_channel` per channel (keeping the loop over all channels unchanged)
- [x] 1.3 Add `#[post("/channels/{channel}/update/")]` handler in `src/handlers/channels.rs` that resolves the channel by slug via `Channel::read_by_id_or_slug`, spawns `update_channel(pool.clone(), channel.id)` on the actix runtime, and returns `CResponse::ok` immediately; return an error response when the channel is not found
- [x] 1.4 Register the new route in `src/handlers/mod.rs` inside the `RequireSession` scope
- [x] 1.5 In the `create` handler, after `Channel::new` succeeds, spawn `update_channel(pool.clone(), channel.id)` so a new channel refreshes immediately
- [x] 1.6 `cargo build` passes with no warnings/errors

## 2. Frontend — refresh control

- [x] 2.1 Add `refreshChannel(slug)` to `src/lib/api/client.ts` issuing `POST /api/1.0/channels/{slug}/update/`
- [x] 2.2 Add a "Refresh" button (with refresh icon, secondary/ghost `AppButton`) to `EpisodesView.vue`, positioned in the page header or next to the search bar
- [x] 2.3 Resolve the channel slug on the episodes page (from the first episode's `channel_slug`, or a channel lookup when the list is empty) so the button targets the viewed channel
- [x] 2.4 Wire the button: set a `refreshing` state, call `api.refreshChannel`, show a success notification ("Channel update started") or error notification, and clear the state
- [x] 2.5 Update the channel creation flow in `ChannelsView.vue`/`AddChannelDialog.vue` success notification to indicate the channel is being updated (per the vue3-spa delta)

## 3. Verification

- [x] 3.1 `pnpm lint`, `pnpm run build`, and `pnpm test` pass in `frontend/`
- [x] 3.2 Smoke-test with the running backend: create a channel and confirm its update starts immediately (episodes appear without waiting for the sleep cycle); call the update endpoint for an existing channel and confirm new episodes are fetched
