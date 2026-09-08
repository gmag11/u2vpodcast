## Purpose

Defines that long-running update operations — the periodic yt-dlp upgrade and the manual full-channel refresh — never block an async runtime thread or an HTTP request handler, and do not stack unbounded background work.

## ADDED Requirements

### Requirement: The yt-dlp auto-update is asynchronous

The periodic `Ytdlp::auto_update` SHALL run through `tokio::process::Command` (or equivalent non-blocking wait): while pip downloads/installs, no runtime thread SHALL be blocked in a synchronous wait.

#### Scenario: Slow pip upgrade does not block the worker
- **WHEN** the worker runs a slow `pip install --upgrade yt-dlp`
- **THEN** other futures on the runtime continue to make progress (no whole-thread blocking)

### Requirement: Manual full sync returns immediately

`GET /api/1.0/options/update/` SHALL acknowledge the request and SHALL NOT wait for all channels to finish processing. The refresh SHALL run in the background; progress and outcome SHALL be observable through per-channel sync status fields (`last_sync_at`, `last_sync_ok`, `last_sync_error`).

#### Scenario: Full sync request returns promptly
- **WHEN** a client triggers a full sync while channels are being refreshed
- **THEN** the endpoint responds immediately and the channels continue updating in the background

### Requirement: No unbounded background task stacking

A new manual full sync SHALL NOT start while a previous manual full sync is still running; overlapping attempts SHALL be rejected with a clear error (e.g. `409 Conflict`). The scheduled worker loop is exempt from this guard.

#### Scenario: Second forced full sync is rejected
- **WHEN** a full sync is already in progress and a second request arrives
- **THEN** the second request is refused with a non-2xx status and a message indicating a sync is running