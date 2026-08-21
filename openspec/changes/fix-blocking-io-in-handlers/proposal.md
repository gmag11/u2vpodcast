## Why

`YTInfo::new` performs a fully synchronous `ureq::get(url).call()` with no request timeout, executed directly inside async handlers (`POST /channels/` and `POST /channels/{slug}/image/`) on tokio worker threads. The server runs with 2 workers (`main.rs:262`) and a 2-connection DB pool. Two concurrent slow or hung YouTube fetches block both worker threads, freezing the whole API including login and session endpoints (self-inflicted DoS from normal UI actions).

## What Changes

- The YouTube metadata fetch SHALL not block async runtime threads: wrap it in `spawn_blocking` (or move to an async HTTP client).
- Add an explicit request timeout so hung upstreams cannot stall threads indefinitely.
- Keep the YTInfo public behavior (title/description/image extraction) unchanged.

## Capabilities

### New Capabilities

- `channel-metadata-fetch`: Defines that HTTP metadata fetching is non-blocking and time-bounded in the async request path.

### Modified Capabilities

(none)

## Impact

- `src/models/ytinfo.rs`, call sites in `src/models/channel.rs` (`Channel::new`, `Channel::update_image`).
- No frontend change.
- Regression guard: re-analysis against `docs/bug-review-2026-08-21.md`; no new bugs (e.g. still no data corruption on timeout errors, error handling preserved).