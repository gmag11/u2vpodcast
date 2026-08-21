## Why

Every page load hotlinks each channel's cover image straight from YouTube's servers (`channel.image` points at `yt3.googleusercontent.com`). A dashboard with many cards opens many connections to YouTube just by rendering the page — wasteful and exactly the kind of burst the connection throttle is meant to prevent. Serving images from a local cache removes that per-page-load traffic entirely.

## What Changes

- Cache each channel's cover image locally when the channel metadata is (re)fetched: on channel creation, on manual image refresh, and during each worker sync/update of the channel.
- The channel `image` field exposed by the API points to the **local** cached copy, not the remote YouTube URL, so the SPA never opens a YouTube connection for images. The local URL is stable per channel (derived from the slug).
- The cached copy is refreshed (re-downloaded) on every channel update, replacing the stored file out-of-band from the HTTP request.
- The cache is served by the app as static content, without auth (cover images are public), consistent with the existing static SPA serving.
- Image downloads (re)use the channel metadata fetch path, so they naturally fall under the single-connection YouTube throttle (`limit-youtube-concurrency`) once that change is implemented.

## Capabilities

### New Capabilities

- `channel-image-cache`: Defines the local cache for channel cover images and the refresh cadence tied to channel updates.

### Modified Capabilities

(none)

## Impact

- New cache directory + static route serving it (mirrors the existing `audios`/`html` pattern; e.g. `images/` with per-slug files).
- `src/models/ytinfo.rs` / `src/models/channel.rs` (download + store bytes), `src/handlers/channels.rs` (refresh_image writes cache), worker `update_channel` (refresh image cache on sync), `src/models/channel.rs` `image` field semantics (API now returns local URL).
- Frontend: no change needed (it renders `channel.image`; the value becomes local). CSP/CORS: the yt3 allowlist entry may become unnecessary once hotlinking stops (can stay as harmless fallback during transition).
- Regression guard applies after implementation: re-analysis against `docs/bug-review-2026-08-21.md`; the cache must not introduce new bugs (no serving of foreign/unauthorized file content, no stale-image-served-as-current regressions, page still renders when cache missing).