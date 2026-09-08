## 1. Non-Blocking Fetches

- [x] 1.1 Wrap the `ureq` metadata request in `spawn_blocking` (or move call sites in `src/models/channel.rs` to spawn the blocking work), keeping the `async` signatures unchanged
- [x] 1.2 Add an explicit timeout to the HTTP request so hung upstreams fail in bounded time through the existing error path

## 2. Verification & Regression

- [x] 2.1 Build and run; add a channel and refresh a cover image with a healthy upstream (unchanged behavior) and verify concurrent requests stay responsive
- [x] 2.2 Run the test suite; re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #5 is resolved and no new bugs were introduced