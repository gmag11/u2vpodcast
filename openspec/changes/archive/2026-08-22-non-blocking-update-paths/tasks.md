## 1. Async yt-dlp auto-update

- [x] 1.1 Replace `std::process::Command` with `tokio::process::Command` in `Ytdlp::auto_update` and await the wait
- [x] 1.2 Preserve exit-code `success()` handling and error mapping
- [x] 1.3 Remove the now-unused `StdCommand` import

## 2. Non-blocking manual full sync

- [x] 2.1 Change `options::update` (`src/handlers/options.rs`) to spawn `do_the_work(&pool)` via `actix_web::rt::spawn` and return `CResponse::ok` immediately
- [x] 2.2 Add an overlap guard (e.g. `AtomicBool`/tokio Mutex try-lock) so a previous manual full sync still running rejects a new one with `409` instead of stacking background tasks
- [x] 2.3 Log start/completion of the background full sync

## 3. Verification

- [x] 3.1 `GET /options/update/` returns promptly (~instant) while a full sync runs; channel `last_sync_*` fields update during the sync
- [x] 3.2 Worker loop with a slow pip update does not block: overall request latency for unrelated endpoints stays flat while `auto_update` runs
- [x] 3.3 Overlap guard: calling the endpoint twice quickly returns `409` on the second call
- [x] 3.4 Check the SPA manual-refresh flow (polling completion via per-channel sync status); adapt frontend if it assumed synchronous completion
- [x] 3.5 Full test suite passes