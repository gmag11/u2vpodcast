## 1. Throttle Infrastructure

- [ ] 1.1 Add a shared throttle module: `tokio::sync::Semaphore::new(1)` in a `OnceLock` plus a cooldown `Duration` (default 3s) from a new optional `config.yml` key (e.g. `cooldown_seconds`); expose a `#[must_use]` RAII guard that holds the permit through the work **and** the post-connection cooldown, releasing on drop
- [ ] 1.2 Add the config key to `Config` with serde default (absent → default cooldown) and initialize the `OnceLock` at startup

## 2. Wire YouTube Connections

- [ ] 2.1 In `YTInfo::new` (metadata + cover images), acquire the throttle guard before the `spawn_blocking` fetch and keep it held until after the fetch and cooldown
- [ ] 2.2 Wrap yt-dlp executions in the worker (`process_channel` download runs and `Ytdlp::auto_update`) with the same throttle guard so they serialize with metadata/image fetches

## 3. Verification & Regression

- [ ] 3.1 Force a refresh of several channels at once: assert strictly sequential yt-dlp runs separated by the configured cooldown, with no overlapping timestamps
- [ ] 3.2 Fire concurrent channel creates/cover refreshes: assert metadata fetches never overlap; assert a failed connection still enforces the cooldown; assert the API (login/status/other endpoints) stays responsive during a held slot
- [ ] 3.3 Unit-test the throttle guard (tiny cooldown) for release-on-success, release-on-error and release-on-panic; run the full test suite
