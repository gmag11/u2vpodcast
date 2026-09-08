## 1. Throttle Infrastructure

- [x] 1.1 Add a shared throttle module: `tokio::sync::Semaphore::new(1)` in a `OnceLock` plus a cooldown `Duration` (default 3s) from a new optional `config.yml` key (e.g. `cooldown_seconds`); expose a `#[must_use]` RAII guard that holds the permit through the work **and** the post-connection cooldown, releasing on drop
- [x] 1.2 Add the config key to `Config` with serde default (absent → default cooldown) and initialize the `OnceLock` at startup

## 2. Wire YouTube Connections

- [x] 2.1 In `YTInfo::new` (metadata + cover images), acquire the throttle guard before the `spawn_blocking` fetch and keep it held until after the fetch and cooldown
- [x] 2.2 Wrap yt-dlp executions in the worker (`process_channel` download runs and `Ytdlp::auto_update`) with the same throttle guard so they serialize with metadata/image fetches

## 3. Verification & Regression

- [x] 3.1 Cover by integration test (`throttle_youtubedl_integration`): 4 concurrent `Ytdlp::download` runs against a fake `yt-dlp` that logs timestamps — strictly sequential (`s`/`e` alternation, peak concurrency 1) with each gap ≥ configured cooldown. Live spot-check on a real deployment remains available.
- [x] 3.2 Cover by tests: `metadata_throttle_tests::concurrent_metadata_fetches_never_overlap` (4 concurrent `YTInfo::new` against a local HTTP server tracking peak concurrency → 1); `cooldown_holds_the_slot_after_error` (failed connection still enforces the cooldown); `independent_async_work_progresses_while_slot_held` (other endpoints/async work stay responsive while the slot is held).
- [x] 3.3 Unit-tested the throttle guard (tiny cooldowns, isolated local slots): release-on-success, release-on-error, release-on-panic, wait-until-cooldown, no-overlap serialization; full test suite green (60 tests).
