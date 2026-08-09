## Why

Channels are currently addressed in every URL by their numeric database id (e.g. `/channels/1/feed.xml`, `/media/1/<yt_id>.mp3`, `/api/1.0/channels/1/episodes/`, `/app/1`). The numeric id is opaque, gives no hint of which channel it is, and makes feed/media URLs unreadable. The operator wants each channel addressed instead by a stable slug derived from its title (e.g. "Confesiones de Gasolinera" → `confesiones_de_gasolinera`), so every URL is self-describing: `/channels/confesiones_de_gasolinera/feed.xml`, `/media/confesiones_de_gasolinera/<yt_id>.mp3`, etc.

## What Changes

- **Add a `slug` column to the `channels` table** (UNIQUE, NOT NULL). The slug is derived from the channel's YouTube title at creation time, stored immutably, and never recomputed (so a YouTube rename does not break feed URLs).
- **Slugify rule**: lowercase, normalize unicode accents to ASCII, replace any run of non-alphanumeric characters with a single underscore, trim leading/trailing underscores. Example: "Confesiones de Gasolinera" → `confesiones_de_gasolinera`.
- **Uniqueness on collision**: if two channels slugify to the same string, append `-2`, `-3`, … until unique.
- **Migrate existing rows and audio directories at startup**: backfill `slug` for existing channels from their current title; rename the existing `/app/audios/{id}/` directory to `/app/audios/{slug}/` for each channel. This is a **BREAKING** change for podcast clients subscribed to the old numeric URLs — they must be re-subscribed to the new slug URLs.
- **Address channels by slug in:**
  - Feed URL: `/channels/{slug}/feed.xml` (was `/channels/{channel_id}/feed.xml`).
  - Media URL: `/media/{slug}/{yt_id}.mp3` (was `/media/{channel_id}/{yt_id}.mp3`).
  - Audio storage: `/app/audios/{slug}/` (was `/app/audios/{channel_id}/`).
  - JSON API: `/api/1.0/channels/{slug}/` and `/api/1.0/channels/{slug}/episodes/`.
  - SPA routes: `/app/{slug}` (channel detail page).
- **The Channel JSON response includes the `slug` field** so the frontend can build feed/media links.
- **The JSON API accepts either the numeric id OR the slug** for the channel path param during a transition window is NOT offered: POST `/api/1.0/channels/` (create) returns the new channel with its slug; channel CRUD and episodes look up by slug. Internally, the numeric `id` remains the primary key and the episodes `channel_id` FK stays numeric.

## Capabilities

### New Capabilities
- `channel-slugs`: generation, storage, uniqueness, and immutability of a per-channel slug derived from its title, used as the public address for the channel across all URLs.

### Modified Capabilities
- `rss-feeds`: the feed URL and the enclosure URL now use the channel's slug instead of its numeric id.

## Impact

- **DB migration**: new `slug` column (UNIQUE, NOT NULL) on `channels`; backfill from title for existing rows.
- **Filesystem migration**: rename `/app/audios/{id}/` → `/app/audios/{slug}/` for each existing channel at startup.
- **Code**: `src/models/channel.rs` (slug field, `read_by_slug`, slugify helper, uniqueness), `src/handlers/feed.rs` (route `{slug}`), `src/handlers/channels.rs` + `episodes.rs` (route `{slug}`), `src/utils/worker.rs` (audio dir uses slug), `src/handlers/mod.rs` (route patterns), `src/models/episode.rs` (no change to FK; media URL resolution uses channel slug).
- **Frontend**: `frontend/src/lib/components/ChannelCard.svelte`, `frontend/src/lib/components/EpisodeCard.svelte`, `frontend/src/routes/[id]/+page.svelte` and `+page.ts`, `frontend/src/routes/+page.svelte` — link/feed/media/SPA routes use `channel.slug` instead of `channel.id`.
- **Dependencies**: possibly `slug` crate (Rust) for slugification; or hand-rolled since `regex` is already a dependency. Frontier: reuse `regex` to avoid a new crate.
- **BREAKING**: old numeric feed/media/SPA URLs stop working. Podcast clients must be re-subscribed to the new slug URLs. No redirect layer from old to new.