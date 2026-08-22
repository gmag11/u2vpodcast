## Why

Every operation that talks to YouTube — metadata reads, cover images, and each yt-dlp execution — opens its own connection with no global limit or pause. A forced refresh of many channels (or a burst of concurrent creates) fires a flurry of simultaneous requests that YouTube can treat as bot behavior and reject. The app needs a single-connection throttle: at most one ongoing YouTube connection at any time, plus a short cooldown after each one.

## What Changes

- Introduce a global YouTube access throttle shared by all YouTube-bound operations.
- Serialize every YouTube connection: metadata fetch (`YTInfo::new`), cover image fetch (same path), and every yt-dlp execution (per-channel downloads and the periodic update check). At most one is in flight at any instant, even when several channels are refreshed at once (scheduled worker, forced "update all", or concurrent manual requests).
- After each connection completes (success or failure) the throttle holds a cooldown pause before the next connection may start.
- Make the cooldown configurable with a sensible default; no code path is left able to bypass the throttle.

## Capabilities

### New Capabilities

- `youtube-throttling`: Defines the single-connection policy with post-connection cooldown for all outbound YouTube traffic.

### Modified Capabilities

(none)

## Impact

- `src/models/ytinfo.rs` (metadata/image fetch), `src/utils/worker.rs` (yt-dlp execution paths), call sites in `src/models/channel.rs`.
- New shared throttle utility (global semaphore + cooldown, tokio-based).
- Optional `config.yml` keys for the cooldown (defaults apply when absent).
- Expected: longer wall-clock for multi-channel refreshes (serialized + cooldown). No API contract change.