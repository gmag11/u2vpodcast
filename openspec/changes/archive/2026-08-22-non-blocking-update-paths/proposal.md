## Why

Two update paths block threads they should not:

1. `Ytdlp::auto_update` (`src/models/ytdlp.rs:88`) runs `StdCommand::wait()` — a synchronous syscall wait — inside the async worker loop's tokio task. `pip install --upgrade yt-dlp` can take tens of seconds to minutes; while waiting, a tokio worker thread is fully blocked. Combined with `max_connections(2)` sqlite pool and only a handshake of workers, one slow pip run can stall unrelated work.
2. `GET /api/1.0/options/update/` (`src/handlers/options.rs:41`) calls `do_the_work(&data.pool)` synchronously inside the HTTP handler. A forced full sync of all channels blocks the request for the whole duration (potentially very long with many channels, each awaiting downloads + 20–40s pacing delays), making the caller (and the SPA) time out and holding a pool connection for minutes.

## What Changes

- Convert `Ytdlp::auto_update` to `tokio::process::Command` with async `.wait()`, preserving exit-code handling; the worker loop keeps its overall cadence.
- Make `GET /options/update/` non-blocking: it spawns the refresh as a background task (the same pattern `create`/`update_episodes` already use via `actix_web::rt::spawn`) and returns immediately with the current response shape. The worker loop and per-channel tasks already serialize and report per-channel status through `last_sync_*`; manual full-sync completion is observable via those fields.

## Capabilities

### New Capabilities

- `non-blocking-updates`: Overview all long-running update operations never block an async worker thread or an HTTP request handler.

### Modified Capabilities

(none)

## Impact

- `src/models/ytdlp.rs` (auto_update async), `src/handlers/options.rs` (spawn + return).
- Behavior change for `GET /options/update/`: it no longer streams completion; callers poll channel sync status instead.
- No schema change; `last_sync_*` fields already exist.

## Non-Goals

- No new async job queue/pub-sub; a fire-and-forget spawn on the actix/tokio runtime is sufficient.
- No change to the scheduled worker loop semantics.
- No change to per-channel concurrency (`limit-youtube-concurrency` still governs gaps).