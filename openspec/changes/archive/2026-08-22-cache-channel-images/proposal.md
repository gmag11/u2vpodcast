## Why

Every page load hotlinks each channel's cover image straight from YouTube's servers (`channel.image` points at `yt3.googleusercontent.com`). A dashboard with many cards opens many connections to YouTube just by rendering the page — wasteful and exactly the kind of burst the connection throttle is meant to prevent. Serving images from a local cache removes that per-page-load traffic entirely.

## What Changes

- Cache each channel's cover image locally when the channel metadata is (re)fetched: on channel creation, on manual image refresh, and during each worker sync/update of the channel.
- The channel `image` field exposed by the API points to the **local** cached copy, not the remote YouTube URL, so the SPA never opens a YouTube connection for images. The local URL is stable per channel (derived from the slug).
- **Skip unchanged images:** before downloading, probe the remote URL with an HTTP `HEAD` request and compare its `Content-Length` with the size of the current cached file. If they match (file exists and same size), the download is skipped — most sync cycles cost one cheap header request, not a full image fetch. If `HEAD` fails or reports no `Content-Length`, fall back to a full (bounded) download.
- The cached copy is refreshed (re-downloaded) on every channel update where the size actually changed, replacing the stored file out-of-band from the HTTP request.
- **The cache lives inside an existing Docker volume** so no new volume is required: production stores files under `/app/db/images` (the `db` volume is already mounted at `/app/db`); local development uses a `db/images` (or `images`) fallback. Placing it in the `audios` volume was rejected because channel slugs can legitimately be `images`, which would collide with the audio directory naming.
- **The cache is NOT public:** the images route is protected by the same `SessionOrBasicAuth` middleware as `/media`. The SPA renders `channel.image` from the same origin, so the session cookie travels automatically; unauthenticated requests get `401`. This also matches the feeds, whose cover/enclosure URLs already require credentials.
- Image downloads (re)use the channel metadata fetch path, so they naturally fall under the single-connection YouTube throttle (`limit-youtube-concurrency`) once that change is implemented.

## Capabilities

### New Capabilities

- `channel-image-cache`: Defines the local, authenticated cache for channel cover images, the refresh cadence tied to channel updates, and the size-based skip that avoids redundant downloads.

### Modified Capabilities

(none)

## Impact

- Cache directory inside the existing `db` volume (`/app/db/images` in the container) + an authenticated static route `{url}/images/{filename}` wrapped in `SessionOrBasicAuth` (mirrors the `/media` pattern).
- `src/models/ytinfo.rs` / `src/models/channel.rs` (HEAD probe + download bytes, atomic writes), `src/handlers/channels.rs` (refresh_image writes cache), worker `update_channel` (refresh cache on sync), channel delete (remove the cached file), `src/models/channel.rs` `image` field semantics (API returns local URL).
- Frontend: no change needed (it renders `channel.image`; the value becomes local and same-origin, cookies flow automatically). CSP/CORS: the yt3 allowlist entry may become unnecessary once hotlinking stops (can stay as harmless fallback during transition).
- Feed/podcast clients: cover image URLs now require the same credentials as the feed/enclosures, consistent with the existing `/media` auth.