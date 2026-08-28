## 1. Persistence and Domain Model

- [ ] 1.1 Add reversible SQLite migrations for a one-to-one `sponsorblock_cache` table keyed by `episode_id` with cascade deletion, snapshot JSON, full hash, check/error timestamps, active processed filename, and processed duration; verify both migrations apply and roll back on a temporary database.
- [ ] 1.2 Add SponsorBlock snapshot, segment, and persistence models with atomic read/upsert operations and episode-query integration; verify model tests distinguish never-checked, successful-empty, and non-empty states and cover cascade deletion.
- [ ] 1.3 Add deterministic selected-media helpers that return the active processed filename/duration or original fallback; verify unit tests cover missing rows, empty snapshots, active derivatives, and missing derived files.

## 2. SponsorBlock Retrieval and Normalization

- [ ] 2.1 Implement the bounded SponsorBlock HTTP client for `https://sponsor.ajay.app/api/skipSegments` using `videoID`, `sponsor`, and `skip`, with injectable transport/base URL for tests; verify fixture-server tests cover data, no-segments/404, timeout, malformed JSON, rate-limit, and server-error responses without using the public network.
- [ ] 2.2 Implement filtering, duration clamping, invalid-interval removal, deterministic ordering, and overlap/adjacency merging; verify table-driven unit tests cover unordered, overlapping, touching, out-of-range, non-finite, empty, and irrelevant-category/action inputs.
- [ ] 2.3 Add versioned canonical serialization and SHA-256 hashing, including the required dependency; verify equivalent effective cuts hash identically while changed endpoints, categories, or processing version change the hash.

## 3. Derived MP3 Processing

- [ ] 3.1 Implement complement-interval and ffconcat-manifest generation for the portions retained from the original MP3; verify unit tests cover sponsors at the beginning, middle, end, multiple merged sponsors, and a sponsor covering the full duration.
- [ ] 3.2 Implement FFmpeg stream-copy generation to a unique temporary file, ffprobe duration measurement, validation of exit status/non-empty output, and atomic publication as `{yt_id}.sponsorblock.{hash-prefix}.mp3`; verify an MP3 fixture produces a playable derivative with expected frame-tolerant duration and leaves the original byte-identical.
- [ ] 3.3 Implement shared fetch-normalize-reconcile orchestration so unchanged hashes skip FFmpeg, changed hashes publish before database selection, empty snapshots restore the original, and failed fetch/process/probe operations retain the last valid state; verify focused tests cover every transition and ensure no partial output becomes active.

## 4. Synchronization and File Lifecycle

- [ ] 4.1 Integrate SponsorBlock reconciliation after download and retention processing for every stored episode in the current YouTube selection window, including already-downloaded episodes and recent favorites; verify worker tests show an old favorite outside the window is untouched while a recent existing episode is refreshed.
- [ ] 4.2 Extend retention and orphan cleanup to remove all files owned by an evicted episode, preserve only the database-selected derivative, and delete stale hash versions and temporary artifacts; verify filesystem-backed tests cover active, superseded, interrupted, and cascade-deleted cases.
- [ ] 4.3 Ensure a channel sync records/logs per-episode SponsorBlock failures without aborting downloads or later reconciliations; verify a mixed-success worker test completes the channel sync and retains prior media for the failed episode.

## 5. Backend API and Feeds

- [ ] 5.1 Extend episode list/read payloads with normalized `sponsorblock_segments` and nullable `sponsorblock_hash` without per-episode database queries; verify handler/model tests cover checked, empty, and never-checked episodes.
- [ ] 5.2 Add a session-protected manual refresh endpoint keyed by `yt_id` that uses shared reconciliation and returns the active snapshot; verify endpoint tests cover an old favorite, unchanged hash, changed hash, unknown episode, and failure preserving prior state.
- [ ] 5.3 Update channel, legacy-id, and global feed generation to select active processed filenames and measured durations with original fallback while retaining stable episode GUIDs; verify RSS tests cover processed, empty, missing-file, and mixed global-feed items.
- [ ] 5.4 Confirm the existing media handler serves hash-versioned processed filenames with `HEAD`, full `GET`, conditional requests, and byte ranges; add focused regression tests and verify valid derivatives return the same cache/range semantics as originals.

## 6. Frontend Playback and Refresh

- [ ] 6.1 Extend frontend episode types and API methods for SponsorBlock segments, hashes, and authenticated refresh responses; verify API client tests assert the refresh route and payload parsing.
- [ ] 6.2 Add a reusable original-timeline skip resolver and integrate it with time updates, absolute/relative seeks, scrubber changes, and resume; verify player-store tests cover entering a segment, seeking/resuming inside one, boundary behavior, empty snapshots, and progress persistence after a skip.
- [ ] 6.3 Apply changed refresh snapshots to the loaded episode without reloading its original media URL and no-op identical hashes; verify player tests retain source, playhead, queue, and playback state in both cases.
- [ ] 6.4 Add a localized `Refresh SponsorBlock segments` action to the existing episode-card menu with loading, success, and error feedback; verify component tests cover invocation for an old favorite and live update of the current player snapshot. Do not add frontend SponsorBlock attribution in this change.

## 7. Documentation and Release Metadata

- [ ] 7.1 Add README attribution linking SponsorBlock, its official service, and CC BY-NC-SA 4.0, state that segment data is transformed into derived audio cuts, and note the non-commercial licensing constraint; verify all links and wording are present while frontend attribution remains documented as pending follow-up work.
- [ ] 7.2 Document synchronization, original/processed filenames, failure fallback, manual refresh, and frame-level cut precision for operators; verify documentation matches the implemented API route and media naming.
- [ ] 7.3 Bump the feature release version consistently in `Cargo.toml`, `frontend/package.json`, and `docker-bake.hcl`; verify all three values and the `vX.Y.Z` image tag match.

## 8. End-to-End Verification

- [ ] 8.1 Run focused Rust tests for SponsorBlock client, normalization, persistence, processing, worker selection, cleanup, handlers, media ranges, and feeds; verify all pass without public SponsorBlock network access.
- [ ] 8.2 Run the complete Rust test suite on Linux and the complete frontend Vitest/typecheck/build pipeline; verify all commands pass and existing playback, progress, queue, retention, and feed behavior has no regressions.
- [ ] 8.3 Perform a container smoke test with a fixture or known video: synchronize, verify original and hash-versioned files, inspect processed duration, exercise `Range`, confirm feed selection, play/seek/resume through a sponsor interval, change the snapshot, and verify atomic regeneration plus stale-file cleanup.