## Context

**auto_update blocking:** `src/models/ytdlp.rs` imports `std::process::Command as StdCommand` precisely for `auto_update`, while the download/list paths use `tokio::process::Command`. The async worker (spawned from `main`, running in the runtime's worker threads) calls `Ytdlp::auto_update().await` each cycle. `StdCommand::spawn().wait()` blocks the calling thread until pip finishes — no .await, no yielding. With a runtime whose threads also handle actix workers' futures, a multi-minute pip run blocks fair co-scheduled work. It also runs every cycle by design; making it async avoids the blocking but not the frequency (frequency is out of scope).

**Synchronous full-sync endpoint:** `GET /options/update/` invokes `do_the_work` inline. `do_the_work` iterates all active channels, and for each spawns a task but `.await`s the join (sequential per-channel), each `process_episode` running a download + 20–40s pacing sleep. So one forced refresh holds the handler future for the entire pipeline of all channels; the SPA request times out; a pool connection is pinned for minutes. The existing per-channel refresh endpoints (`POST /channels/{id}/update/`) already fire-and-forget via `actix_web::rt::spawn` and return the channel object; completion status is readable through `Channel::last_sync_*`.

## Goals / Non-Goals

**Goals:**
- No tokio worker thread is blocked by a synchronous `wait()` in the update loop.
- `GET /options/update/` returns promptly and the refresh continues in the background.

**Non-Goals:**
- No job/task registry, no progress bar API, no websocket.
- No change to the scheduled loop's cadence or to `sleep_time` semantics.
- No change to download pacing (that belongs to `limit-youtube-concurrency`).

## Decisions

- **Async pip:** switch `auto_update` to `tokio::process::Command`; `.spawn()?.wait().await` yields correctly, then `success()` as today. Remove the `StdCommand` import. Error mapping stays (`Error::default`).
- **Background full-sync:** in `options::update`, replace the direct `do_the_work(...).await` with:
  ```rust
  let pool = data.pool.clone();
  actix_web::rt::spawn(async move {
      if let Err(e) = do_the_work(&pool).await { error!("Full sync failed: {e}"); }
  });
  ```
  and return `CResponse::ok(session, "")` immediately. Mirrors the `create`/`update_episodes` pattern.
- **Completion observability:** after the change, `GET /options/update/` acknowledges the request; actual results surface in channel `last_sync_at`/`last_sync_ok`/`last_sync_error`. The SPA already reads those fields for per-channel sync age.
- **Rollback safety:** both changes degrade gracefully; a revert returns to today's behavior.

## Risks / Trade-offs

- [SPA may expect the synchronous behavior/content] → Frontend check required: if the SPA triggers full-refresh and then reads results, it should poll channel sync status (fields already present). Document the response semantic change in the change notes.
- [Unbounded background tasks if user hammers the endpoint] → Same exposure already exists via per-channel update endpoints and the worker; accepted. A per-process "sync in progress" mutex guard is a cheap add-on: acquire a `tokio::sync::Mutex` or `AtomicBool` (try-lock) and reject overlapping manual full syncs with a `409`.
- [pip updates interleaving with worker] → `auto_update` is part of the worker loop and multitasks correctly once async; yt-dlp binary swapping mid-job remains a pre-existing risk (downloads use the path resolved at runtime).

## Migration Plan

1. Make `auto_update` async (tokio Command).
2. Make `/options/update/` fire-and-forget; add optional overlap guard.
3. Test: forced full sync returns ~instantly while channels update afterward (observe `last_sync_at`); worker loop no longer blocks a thread during pip (log cadence).
4. Frontend verification: SPA flow for manual refresh completes via status polling.

## Open Questions

- Should `/options/update/` return `202 Accepted` instead of `200 Ok` to signal background execution? (Default: keep `CResponse::ok` session shape; revisit if SPA needs to distinguish.)