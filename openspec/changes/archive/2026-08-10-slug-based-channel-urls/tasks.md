## 1. Dependency and DB migration

- [x] 1.1 Add `deunicode = "1"` to `Cargo.toml` for ASCII-folding accented titles.
- [x] 1.2 Create migration `migrations/<ts>_add_slug.up.sql` with `ALTER TABLE channels ADD COLUMN slug TEXT;` and a matching `<ts>_add_slug.down.sql` that drops the column.

## 2. Backend — Channel model

- [x] 2.1 Add a `slug: String` field to the `Channel` struct in `src/models/channel.rs` and update `from_row` to read it.
- [x] 2.2 Implement `fn slugify(title: &str) -> String`: `deunicode` to ASCII, lowercase, `regex` replace `[^a-z0-9]+` with `_`, trim leading/trailing `_`. Return `channel-{id}` fallback when the result is empty (used post-insert).
- [x] 2.3 In `Channel::new`: fetch the `YTInfo` title, slugify it, ensure uniqueness (append `-2`, `-3`, … by querying existing slugs), INSERT with the slug column, and on the empty-slug case update the row to `channel-{id}`. Confirm the returned row carries the final `slug`.
- [x] 2.4 Add `pub async fn read_by_slug(pool: &SqlitePool, slug: &str) -> Result<Channel, Error>` selecting `WHERE slug = $1`.
- [x] 2.5 Update `Channel::delete` to remove the audio directory by slug: `format!("{}/{}", FOLDER, &channel.slug)` (the `FOLDER` constant moves to `channel.rs` or is shared). Keep the existing `DELETE … RETURNING *` SQL. (Dir removal is done by the handler in 5.2; the model `delete` SQL stays keyed by id.)

## 3. Backend — Startup migration

- [x] 3.1 Add `pub async fn migrate_slugs(pool: &SqlitePool, audio_folder: &str) -> Result<(), Error>` in `src/models/channel.rs` that: for each channel with `slug IS NULL`, slugify its title with uniqueness and `UPDATE channels SET slug = ? WHERE id = ?`; then for each channel, `tokio::fs::rename("{audio_folder}/{id}", "{audio_folder}/{slug}")` when the `{id}` dir exists and the `{slug}` dir does not. Log every rename at INFO. Idempotent.
- [x] 3.2 In `src/main.rs`, after `Migrator::new(...).run(&pool)` and `User::default`, call `Channel::migrate_slugs(&pool, "/app/audios").await` before the worker loop spawns.

## 4. Backend — Worker

- [x] 4.1 In `src/utils/worker.rs`, replace every `format!("{}/{}/{}.mp3", FOLDER, &channel.id, ...)` with `format!("{}/{}/{}.mp3", FOLDER, &channel.slug, ...)` (`process_channel`'s `create_dir_all`, `process_episode`, and `clean_channel`).

## 5. Backend — Handlers (feed + API id-or-slug + episodes payload)

- [x] 5.1 In `src/handlers/feed.rs`: change the route to `/channels/{slug}/feed.xml`; `get_feed` reads `Channel::read_by_slug` and builds the enclosure URL `{url}/media/{slug}/{yt_id}.mp3`.
- [x] 5.2 In `src/handlers/channels.rs`: use a single path param `{channel}` for `read`, `update`, `delete`; parse `Path<String>` and call `Channel::read_by_id_or_slug`; `update` resolves the existing channel to set its numeric id before applying the `UpdateChannel` body; `delete` removes `{FOLDER}/{channel.slug}` then deletes by id. Remove the old query-based delete and body-id-only update.
- [x] 5.3 In `src/handlers/episodes.rs`: route stays under `/channels/{channel}/episodes/`; resolve the channel by id-or-slug, then call `Episode::read_episodes_for_channel(&pool, channel.id)` and populate each episode's `channel_slug` from the channel's slug.
- [x] 5.4 Add a `channel_slug: String` field (serde default) to `Episode` in `src/models/episode.rs`, set it in the episodes handler, so the frontend can build `/media/{channel_slug}/{yt_id}.mp3`.

## 6. Frontend

- [x] 6.1 Keep the route folder `frontend/src/routes/[id]`; update `+page.ts` to also return `channel_slug` (derived from the first episode's `channel_slug`) and `+page.svelte` to use `data.channel_slug` in the feed link.
- [x] 6.2 In `frontend/src/lib/components/ChannelCard.svelte`: `/app/{channel.id}` stays; feed link uses `/channels/${channel.slug}/feed.xml`.
- [x] 6.3 In `frontend/src/lib/components/EpisodeCard.svelte`: build the media URL as `{base_endpoint}/media/${episode.channel_slug}/${episode.yt_id}.mp3`.
- [x] 6.4 Update `frontend/src/lib/types.ts` to add `slug: string` to the Channel type and `channel_slug: string` to the Episode type; add `slug: ''` to the new-channel default object in `+page.svelte`.
- [x] 6.5 In `frontend/src/routes/+page.svelte`, update `PUT` and `DELETE` to use the slug in the path: `/api/1.0/channels/${channelToUpdate.slug}/` and `/api/1.0/channels/${channelToDelete.slug}/`.

## 7. Verification

- [x] 7.1 `cargo build` in the container and redeploy; confirm the image runs. Confirmed by operator.
- [x] 7.2 On first start after upgrade, confirm the logs show the slug backfill + audio dir renames for existing channels. Confirmed by operator.
- [x] 7.3 `curl -i -u admin:<pass> /channels/<slug>/feed.xml` for a channel and confirm the feed loads with items only for that channel, ordered newest first, and enclosure URLs `{url}/media/<slug>/<yt_id>.mp3`. Confirmed by operator.
- [x] 7.4 Confirm `curl -i -u admin:<pass> /media/<slug>/<yt_id>.mp3` returns `200` with the MP3 body. Confirmed by operator.
- [x] 7.5 Confirm the SPA: channel list renders, channel card links go to `/app/<id>`, the episodes page loads episodes by id, and the audio player fetches `/media/<slug>/<yt_id>.mp3`. Confirmed by operator.
- [x] 7.6 Confirm the JSON API works with both id and slug: `GET /api/1.0/channels/1/` and `GET /api/1.0/channels/<slug>/` both return the channel; `GET /api/1.0/channels/1/episodes/` and `GET /api/1.0/channels/<slug>/episodes/` both return the episodes with `channel_slug`. Confirmed by operator.
- [x] 7.7 Confirm the old numeric feed/media URLs (`/channels/1/feed.xml`, `/media/1/...`) return 404 / are gone (breaking change accepted). Confirmed by operator.

## 8. Release notes

- [x] 8.1 Note in the next tag's release notes that channel URLs now use a slug derived from the channel title, that existing podcast clients must be re-subscribed to the new slug URLs, and that existing audio directories are migrated automatically on the first start after upgrade. Recorded for the next release tag (no CHANGELOG file exists; the project documents releases via git tags).