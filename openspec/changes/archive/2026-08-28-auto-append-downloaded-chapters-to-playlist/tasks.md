## 1. Core Implementation

- [x] 1.1 In `src/utils/worker.rs`, capture the result of `Episode::new(...)` (replace `let _ =` with `let episode = ... .await?;`) so the persisted episode's `id` is available at the download-completion point
- [x] 1.2 After the episode persists, call `PlaylistItem::add(pool, episode.id).await` from the worker, reusing the existing append/dedupe semantics
- [x] 1.3 Handle the append as best-effort: treat the "episode already in playlist" case as a no-op (log at debug/warn, do NOT fail the run) and log other failures at `error!`/`warn!` before continuing the sync run — never use `?` on this call
- [x] 1.4 Make sure the append happens only after the retention-floor check passes (i.e. inside the success path after `Episode::new`, never for discarded episodes)

## 2. Tests & Verification

- [x] 2.1 Confirm existing playlist dedupe coverage still holds (`add_rejects_duplicates_with_conflict` in `src/models/playlist.rs`) and add/extend a test asserting a second add of the same episode leaves a single entry unchanged
- [x] 2.2 Run `cargo build` (or `cargo check`) and `cargo clippy` cleanly
- [x] 2.3 Manual smoke test: run a channel sync, then `GET /playlist/` and verify each newly downloaded episode appears at the end in download order; reorder one episode and remove another, then resync and verify the reordered episode keeps its position and the removed episode is not re-appended