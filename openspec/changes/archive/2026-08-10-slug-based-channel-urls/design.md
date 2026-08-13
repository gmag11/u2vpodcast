## Context

See `proposal.md - Why` for the motivation. Relevant current state:

- `channels` table (`migrations/20240319181850_channels.up.sql`) has `id INTEGER PRIMARY KEY AUTOINCREMENT`, no slug column.
- `Channel` struct (`src/models/channel.rs:34`) has fields `id, url, title, active, description, image, first, max, created_at, updated_at` — no slug.
- `Channel::read` (channel.rs:115) looks up by numeric `id`; `Channel::new` (channel.rs:88) inserts without a slug.
- `episodes` table has `channel_id INTEGER` FK; `Episode::read_episodes_for_channel(pool, channel_id)` filters by numeric id.
- Feed route (`src/handlers/feed.rs:24`): `/channels/{channel_id}/feed.xml`, `get_feed` reads `Channel::read(&pool, channel_id)` and builds enclosure `{url}/media/{channel_id}/{yt_id}.mp3`.
- Media served from `/media/{channel_id}/{yt_id}.mp3` via `af::Files::new("", "./audios")` inside `web::scope("/media")` (after the route-protection change).
- Worker (`src/utils/worker.rs`) stores audios at `format!("{}/{}/{}.mp3", FOLDER, &channel.id, &ytvideo.id)` with `FOLDER = "/app/audios"`; `clean_channel` deletes `format!("{}/{}/{}.mp3", FOLDER, &channel.id, episode.yt_id)`.
- API routes (`handlers/channels.rs`, `handlers/episodes.rs`): `#[get("/channels/{channel_id}/")]`, `#[get("/channels/{channel_id}/episodes/")]` — numeric path param.
- SPA: `ChannelCard.svelte:20` `/app/{channel.id}`, `:45` `${base_endpoint}/channels/${channel.id}/feed.xml`; `EpisodeCard.svelte:32` `{base_endpoint}/media/{episode.channel_id}/{episode.yt_id}.mp3`; `frontend/src/routes/[id]/+page.ts` fetches `/api/1.0/channels/${params.id}/episodes/` and returns `channel_id: Number(params.id)`; the route folder is `[id]`.
- `regex` is already a dependency (`Cargo.toml`).

## Goals / Non-Goals

**Goals:**
- Every channel has a UNIQUE, NOT NULL, immutable `slug` derived from its YouTube title at creation.
- Feed, media, API, and SPA URLs address a channel by its slug.
- Existing channels and their audio directories are migrated to the slug scheme at startup.
- Old numeric URLs are NOT supported (clean break, per proposal).

**Non-Goals:**
- Backwards compatibility / redirects from old numeric URLs to slug URLs (breaking change, accepted).
- Slugs derived from a user-editable field (the slug is derived from the YouTube title only, at creation).
- Changing the `episodes.channel_id` numeric FK.
- Per-episode slugs; only channel-level addressing changes.

## Decisions

### Decision 1: Store the slug in a new `slug` column, not compute on the fly

Add `slug TEXT NOT NULL UNIQUE` to the `channels` table. Compute it once at channel creation from the fetched YouTube title, persist it, and never recompute.

**Why**: a YouTube channel rename would silently break every feed/media URL if the slug were computed on the fly. Persisting keeps URLs stable and makes lookups a single indexed equality query.

**Alternative considered**: Compute slug from title per request. Rejected: breaks URLs on YouTube rename and complicates uniqueness.

### Decision 2: Slugify with a small hand-rolled helper over `regex` (no new crate)

Implement `fn slugify(title: &str) -> String` in `src/models/channel.rs`:
1. `unicode` NFKD normalize and drop non-ASCII (or use `String::from(...).chars().filter_map(|c| char::deunicode(c))` — but `deunicode` is a dependency. Simpler: keep ASCII a-z, 0-9; map accented chars by a small static table, or use `.to_lowercase()` then `regex` replace `[^a-z0-9]+` with `_` and trim `_`). Since `regex` is already a dependency, the helper is: lowercase → `regex::replace_all(r"[^a-z0-9]+", "_")` → trim underscores. Accents: ASCII-fold via `String`'s `to_lowercase` does NOT strip accents. Need a small accent-folding pass or a tiny `deunicode`-style map.

**Decision**: add the `deunicode` crate (small, no_std, pure Rust) for accent→ASCII folding, then lowercase + `regex` collapse non-alphanumeric to `_`. `deunicode` is ~1 small dependency.

**Alternative considered**: hand-roll an accent map. Rejected: incomplete, error-prone for Spanish/other diacritics. `slug` crate rejected: it uses `-` separators and has more opinions; the helper with `deunicode` + `regex` is 6 lines and matches the operator's `_` separator.

### Decision 3: Uniqueness by `-N` suffix at creation

In `Channel::new`, after slugifying, query for existing channels with the same slug or `slug || '-%'`; pick the lowest N ≥ 2 such that the candidate is unused. Insert under that slug. All within a single INSERT in a transaction would be ideal, but SQLite app-level: do a SELECT-then-INSERT; rely on the UNIQUE constraint to catch races and retry with a higher N on conflict.

### Decision 4: Migration runs at startup in `main.rs` after migrations, before the worker

After `Migrator::new(...).run(&pool)` and after `User::default`, run a `Channel::migrate_slugs(&pool, FOLDER)` that:
1. `ALTER TABLE channels ADD COLUMN slug TEXT` (inside a migration file, not runtime — add a new migration).
2. For each channel with `slug IS NULL`: compute slug from title with uniqueness, `UPDATE channels SET slug = ? WHERE id = ?`.
3. For each channel: `tokio::fs::rename("/app/audios/{id}", "/app/audios/{slug}")` if the id dir exists and the slug dir doesn't.

**Why at startup**: idempotent, runs once per deploy, no separate script. The DB column comes from a real migration file (rolled back by the down migration).

**Alternative considered**: a separate `migrate` binary. Rejected: more moving parts; the app already owns startup.

### Decision 5: JSON API accepts id or slug via a single path parameter; SPA stays on id

The JSON API channel routes use a single path parameter `{channel}` (e.g. `/api/1.0/channels/{channel}/`, `/api/1.0/channels/{channel}/episodes/`) that resolves via `Channel::read_by_id_or_slug` (new): parse the value as `i64` → `read(id)`, otherwise `read_by_slug`. `PUT` and `DELETE` use the same id-or-slug path parameter; update resolves the existing channel to set its numeric id before applying the `UpdateChannel` body, and delete removes `/app/audios/{channel.slug}`. The `Channel` and `Episode` structs gain serialized `slug`/`channel_slug` fields so the frontend can build the slug-based feed and media URLs. The SPA keeps routing by id (`/app/{id}`) and calling the API by id — both still work because the API accepts id.

**Why**: the operator wants both addressing schemes for the API (id retained for compatibility, slug added for readability), and the SPA keeps id-based routing to minimize frontend churn while linking feed/media by slug.

**Alternative considered**: switching the API and SPA entirely to slug. Rejected by the operator — id must keep working.

### Decision 6: Feed route and enclosure use slug

`feed.rs`: route `/channels/{slug}/feed.xml`; `get_feed` reads `Channel::read_by_slug`; enclosure URL `{url}/media/{slug}/{yt_id}.mp3`.

### Decision 7: Worker stores audios under the slug directory

`worker.rs`: `process_channel` creates `format!("{}/{}", FOLDER, &channel.slug)`; `process_episode` and `clean_channel` use `format!("{}/{}/{}.mp3", FOLDER, &channel.slug, ...)`. `Channel::delete` (channels.rs) removes `FOLDER/{slug}`.

### Decision 8: SPA links use slug for feed/media, stays on id for routes/API

The SPA route folder stays `frontend/src/routes/[id]`; `+page.ts` keeps fetching `/api/1.0/channels/${params.id}/episodes/` (works — API accepts id) and returns `channel_id` plus `channel_slug` (derived from the first episode's `channel_slug`). `ChannelCard.svelte` uses `channel.slug` for the feed link (`/channels/{slug}/feed.xml`); the channel detail link stays `/app/{id}`. `EpisodeCard.svelte` builds the media URL from `episode.channel_slug` (`/media/{channel_slug}/{yt_id}.mp3`). `+page.svelte` uses `channel.slug` in the PUT/DELETE paths (the API resolves both). The episodes API populates `channel_slug` on each episode (Decision 5).

## Risks / Trade-offs

- **[BREAKING] Old numeric feed/media/API/SPA URLs stop working.** → Mitigation: documented as breaking in the proposal; the operator re-subscribes podcast clients to the new slug URLs. No redirect layer is offered.
- **[Risk] Two channels slugify to the same string.** → Mitigation: `-2`, `-3` suffix at creation (Decision 3). Migration backfill uses the same rule, so existing rows are disambiguated too.
- **[Risk] Migration renames a dir that a running worker is writing to.** → Mitigation: migration runs before the worker loop starts (Decision 4 ordering), so no concurrent access during the rename.
- **[Risk] `channel.slug` is `""` if the YouTube title slugifies to empty (rare).** → Mitigation: fall back to the string form of the numeric id, or reject channel creation. Decision: at creation, if slugify returns empty, fall back to `channel-{id}` (using the freshly inserted id). Documented.
- **[Trade-off] `deunicode` is a new dependency.** Small and pure-Rust. Acceptable to handle Spanish/other diacritics correctly without hand-rolling a table.
- **[Trade-off] Episodes API payload gains `channel_slug`.** Minor, non-breaking for consumers that ignore extra fields.

## Migration Plan

1. Add migration `migrations/<ts>_add_slug.up.sql` — `ALTER TABLE channels ADD COLUMN slug TEXT;` + `UPDATE channels SET slug = ... ` (the backfill is done at startup because it needs the slugify helper, not pure SQL). The down migration drops the column.
2. App startup: run migrations, then `Channel::migrate_slugs` (backfill + rename audios) before the worker loop.
3. Deploy. Existing podcast clients must be re-subscribed to the new slug-based feed URLs.

**Rollback**: revert the change. The `slug` column remains (nullable). Audio dirs are under `{slug}`; to restore the id layout the operator would need to rename them back manually, so rollback is non-trivial — the operator should test on a copy first.

## Open Questions

- Should the operator run a dry-run of the migration (print planned `slug` + rename pairs without touching the FS/DB) before the real one? Deferrable: the migration is idempotent and logs every rename at INFO, so a first start with `RUST_LOG=info` serves as a preview. If a true dry-run is desired, add a `--dry-run` flag later.