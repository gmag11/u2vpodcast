## 1. Media resolution backend

- [x] 1.1 Add an episode media-resolution helper that, given the channel slug and `yt_id`, returns the active `processed_filename` from `sponsorblock_cache` when SponsorBlock is enabled
- [x] 1.2 Extend `serve_media` to accept `Data<AppState>` and classify the requested path: `{yt_id}.original.mp3` → original, plain `{yt_id}.mp3` → processed-or-original resolution, everything else → direct file
- [x] 1.3 Keep the existing path-traversal guard, byte-range, HEAD, ETag, and conditional-request handling derived from the resolved physical file
- [x] 1.4 Pass `AppState` into the `/media` scope handlers in `src/handlers/mod.rs`
- [x] 1.5 Add media handler tests: active processed served at the stable URL, original served when disabled/no derivative/missing derivative, `.original.mp3` always serves the original, hash-versioned route still serves its file, and `404` when nothing exists

## 2. Feed generation

- [x] 2.1 Change `episode_item` so the `<enclosure>` URL is always `{url}/media/{slug}/{yt_id}.mp3`
- [x] 2.2 Keep the `<enclosure>` `length` and `itunes:duration` derived from the served representation (processed when active, otherwise original)
- [x] 2.3 Update feed tests to assert the stable enclosure URL and the processed/original `length` values
- [x] 2.4 Confirm the chapters endpoint still returns translated chapters when the processed representation is served and add or adjust a regression assertion

## 3. Frontend player

- [x] 3.1 Change the player media URL builder in `frontend/src/stores/player.ts` to `/media/{slug}/{yt_id}.original.mp3`
- [x] 3.2 Update the player store tests to expect the `.original.mp3` URL
- [x] 3.3 Confirm episode-card and persistent-player progress tracks, chapter markers, and SponsorBlock skip math still use the original timeline

## 4. Verification

- [x] 4.1 Run `cargo fmt --check`, `cargo clippy --all-targets`, and `cargo test`
- [x] 4.2 Run the frontend lint, test, and build commands
- [x] 4.3 End-to-end check: fetch a channel feed and confirm the enclosure is `{yt_id}.mp3`, download that URL and confirm the processed duration, and confirm the web player loads `.original.mp3` and skips rejected intervals
- [x] 4.4 Verify an old cached feed's `{yt_id}.sponsorblock.{hash}.mp3` URL still downloads
