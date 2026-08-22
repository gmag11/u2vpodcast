## 1. Cache Infrastructure

- [x] 1.1 Add an `images_dir()` helper: `/app/db/images` inside the container (the existing `db` volume — no `docker-compose.yml` change), local `images` fallback in development; ensure the directory exists at startup
- [x] 1.2 Register an authenticated static route `{url}/images/{filename}` scoped to the cache directory, wrapped in the same `SessionOrBasicAuth` middleware as `/media` (401 for unauthenticated requests)

## 2. Cache Population & Refresh

- [x] 2.1 Add a shared image-fetch helper: HTTP `HEAD` probe first (compare `Content-Length` with the current file size → skip when equal), then a bounded GET (timeout + max-size cap, atomic temp-file + rename), storing `{slug}.jpg` and returning the local URL on success while keeping the old file on failure
- [x] 2.2 Populate the cache in `Channel::new` (creation) and `Channel::update_image` (manual refresh), setting `channel.image` to the local URL
- [x] 2.3 Refresh the cached image during worker `update_channel` (each scheduled/forced sync), skipping inactive channels per the `active` flag semantics
- [x] 2.4 Remove the cached image file when a channel is deleted (alongside the audio directory removal)
- [x] 2.5 Ensure the probe + download path uses the same fetch mechanism as `YTInfo::new` so the single-connection throttle (`limit-youtube-concurrency`) covers it once implemented

## 3. Verification & Regression

- [x] 3.1 Verify: page load makes zero requests to YouTube for images (verified in deployment: after channel sync, the web makes no requests to YouTube; local authenticated `/images/{slug}.jpg` serves bytes with a session and returns `401` without one; manual refresh replaces the file; sync refreshes it; failed download keeps the previous image and does not blank `channel.image`)
- [x] 3.2 Verify skip-if-same: covered by integration tests (local HTTP server counting requests: unchanged size → HEAD only, zero GETs; changed size → one GET) plus a real-channel test against `youtube.com/c/atareao` where the unchanged cover re-probe returns Skip
- [x] 3.3 Verify cache route confinement: request outside the cache directory is not served (verified locally: plain and URL-encoded `..` traversal on `/images/*` → 404, mirroring `/media`)
- [x] 3.4 Verify cache persistence: `tests/image_cache_deployment_guard.rs` asserts the cache lives under the existing `db` mount (`/app/db/images`), `docker-compose.yml` gains no volume entries, and failed refreshes keep the previous file (integration test). Container-recreation smoke remains available as an optional live Docker check.
