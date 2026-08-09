## 1. Dependency and DB migration

- [ ] 1.1 Add `deunicode = "1"` to `Cargo.toml` for ASCII-folding accented titles.
- [ ] 1.2 Create migration `migrations/<ts>_add_slug.up.sql` with `ALTER TABLE channels ADD COLUMN slug TEXT;` and a matching `<ts>_add_slug.down.sql` that drops the column.

## 2. Backend — Channel model

- [ ] 2.1 Add a `slug: String` field to the `Channel` struct in `src/models/channel.rs` and update `from_row` to read it.
- [ ] 2.2 Implement `fn slugify(title: &str) -> String`: `deunicode` to ASCII, lowercase, `regex` replace `[^a-z0-9]+` with `_`, trim leading/trailing `_`. Return `channel-{id}` fallback when the result is empty (used post-insert).
- [ ] 2.3 In `Channel::new`: fetch the `YTInfo` title, slugify it, ensure uniqueness (append `-2`, `-3`, … by querying existing slugs), INSERT with the slug column, and on the empty-slug case update the row to `channel-{id}`. Confirm the returned row carries the final `slug`.
- [ ] 2.4 Add `pub async fn read_by_slug(pool: &SqlitePool, slug: &str) -> Result<Channel, Error>` selecting `WHERE slug = $1`.
- [ ] 2.5 Update `Channel::delete` to remove the audio directory by slug: `format!("{}/{}", FOLDER, &channel.slug)` (the `FOLDER` constant moves to `channel.rs` or is shared). Keep the existing `DELETE … RETURNING *` SQL.

## 3. Backend — Startup migration

- [ ] 3.1 Add `pub async fn migrate_slugs(pool: &SqlitePool, audio_folder: &str) -> Result<(), Error>` in `src/models/channel.rs` that: for each channel with `slug IS NULL`, slugify its title with uniqueness and `UPDATE channels SET slug = ? WHERE id = ?`; then for each channel, `tokio::fs::rename("{audio_folder}/{id}", "{audio_folder}/{slug}")` when the `{id}` dir exists and the `{slug}` dir does not. Log every rename at INFO. Idempotent.
- [ ] 3.2 In `src/main.rs`, after `Migrator::new(...).run(&pool)` and `User::default`, call `Channel::migrate_slugs(&pool, "/app/audios").await` before the worker loop spawns.

## 4. Backend — Worker

- [ ] 4.1 In `src/utils/worker.rs`, replace every `format!("{}/{}/{}.mp3", FOLDER, &channel.id, ...)` with `format!("{}/{}/{}.mp3", FOLDER, &channel.slug, ...)` (`process_channel`'s `create_dir_all`, `process_episode`, and `clean_channel`).

## 5. Backend — Handlers (API + feed)

- [ ] 5.1 In `src/handlers/feed.rs`: change the route to `/channels/{slug}/feed.xml`; `get_feed` reads `Channel::read_by_slug` and builds the enclosure URL `{url}/media/{slug}/{yt_id}.mp3`.
- [ ] 5.2 In `src/handlers/channels.rs`: change `{channel_id}` → `{slug}` in the `#[get]`, `#[post]`, `#[put]`, `#[delete]` route macros; parse `Path<String>`; call `Channel::read_by_slug` / resolve-by-slug for update/delete; `delete` removes `{FOLDER}/{slug}`.
- [ ] 5.3 In `src/handlers/episodes.rs`: change the route to `/channels/{slug}/episodes/`; resolve the channel by slug, then call `Episode::read_episodes_for_channel(&pool, channel.id)` (the episodes query still filters by numeric id).
- [ ] 5.4 Include the channel `slug` in the episodes API response (`EpisodeCard` needs it for the media URL): add `channel_slug: String` to the episode JSON payload (resolve from the channel by `episode.channel_id`).

## 6. Frontend

- [ ] 6.1 Rename the route folder `frontend/src/routes/[id]` → `frontend/src/routes/[slug]`. Update `+page.ts` to fetch `/api/1.0/channels/${params.slug}/episodes/` and return `channel_slug: params.slug`. Update `+page.svelte` to use `data.channel_slug` in the feed and media links.
- [ ] 6.2 In `frontend/src/lib/components/ChannelCard.svelte`: `/app/{channel.slug}` and `${base_endpoint}/channels/${channel.slug}/feed.xml`.
- [ ] 6.3 In `frontend/src/lib/components/EpisodeCard.svelte`: build the media URL as `{base_endpoint}/media/${episode.channel_slug}/${episode.yt_id}.mp3`.
- [ ] 6.4 Update `frontend/src/lib/types.ts` to add `slug: string` to the Channel type and `channel_slug: string` to the Episode type.

## 7. Verification

- [ ] 7.1 `cargo build` in the container and redeploy; confirm the image runs.
- [ ] 7.2 On first start after upgrade, confirm the logs show the slug backfill + audio dir renames for existing channels.
- [ ] 7.3 `curl -i -u admin:<pass> /channels/<slug>/feed.xml` for a channel and confirm the feed loads with items only for that channel, ordered newest first, and enclosure URLs `{url}/media/<slug>/<yt_id>.mp3`.
- [ ] 7.4 Confirm `curl -i -u admin:<pass> /media/<slug>/<yt_id>.mp3` returns `200` with the MP3 body.
- [ ] 7.5 Confirm the SPA: channel list renders, channel card links go to `/app/<slug>`, the episodes page loads episodes by slug, and the audio player fetches `/media/<slug>/<yt_id>.mp3`.
- [ ] 7.6 Confirm old numeric URLs (`/channels/1/feed.xml`, `/media/1/...`, `/api/1.0/channels/1/`, `/app/1`) return 404 / are gone (breaking change accepted).

## 8. Release notes

- [ ] 8.1 Note in the next tag's release notes that channel URLs now use a slug derived from the channel title, that existing podcast clients must be re-subscribed to the new slug URLs, and that existing audio directories are migrated automatically on the first start after upgrade.