## Why

Podcast clients such as PodcastAddict identify an episode by its `<guid>` and keep the enclosure URL they stored when the episode first appeared; they do not adopt a new enclosure URL for an already-known episode even after the feed changes it. Because SponsorBlock replaces the feed enclosure with a hash-versioned derivative (`{yt_id}.sponsorblock.{hash}.mp3`), existing subscribers keep downloading and playing the original audio and never hear the processed version, even after deleting and re-downloading the episode. A stable enclosure URL whose resolution happens server-side serves the processed audio without forcing clients to see a new episode.

## What Changes

- Feed items always publish the stable enclosure URL `{url}/media/{slug}/{yt_id}.mp3`, independent of whether a SponsorBlock derivative exists.
- The media route resolves `/media/{slug}/{yt_id}.mp3` to the active SponsorBlock-derived file when SponsorBlock is enabled and a valid derivative exists, and to the original MP3 otherwise.
- A dedicated `/media/{slug}/{yt_id}.original.mp3` route always serves the original, on-disk MP3.
- The web player loads the dedicated original route so its client-side SponsorBlock skipping keeps working against the original timeline.
- Hash-versioned derivative URLs (`{yt_id}.sponsorblock.{hash}.mp3`) remain served unchanged for already-cached feeds and downloads.
- The `<enclosure length>` continues to describe the file actually served at the stable URL (already implemented as the physical selected file).
- No database schema change, no on-disk file rename, and no change to retention or orphan cleanup.

## Capabilities

### New Capabilities
- `media-serving`: server-side resolution of the stable episode media URL to the processed or original audio, plus the dedicated original-media route.

### Modified Capabilities
- `rss-feeds`: enclosure URLs become the stable `{yt_id}.mp3`; duration and length describe the representation actually served at that URL.
- `rss-podcast-chapters`: chapter translation is keyed on the served media representation rather than on the enclosure filename.
- `persistent-audio-player`: the shared player loads `/media/{slug}/{yt_id}.original.mp3` as the original source.
- `vue3-spa`: the playback media URL pattern becomes `/media/{slug}/{yt_id}.original.mp3`.

## Impact

- Code: `src/handlers/feed.rs`, `src/handlers/media.rs`, `src/handlers/mod.rs`, `frontend/src/stores/player.ts` and its tests.
- Specs: `openspec/specs/rss-feeds`, `openspec/specs/rss-podcast-chapters`, `openspec/specs/persistent-audio-player`, `openspec/specs/vue3-spa`, and the new `openspec/specs/media-serving`.
- No database migration, no API payload change, and no change to the stored SponsorBlock cache or derivative filenames.
- Deployment note: clients that already cached a hash-versioned or original enclosure URL keep that URL until the client itself refreshes its local episode record, but the stable URL lets a manual re-download retrieve the processed audio.
