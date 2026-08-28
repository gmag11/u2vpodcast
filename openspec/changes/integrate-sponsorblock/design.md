## Context

See `proposal.md` for motivation. The worker stores original audio at `{audios_dir}/{channel_slug}/{yt_id}.mp3`; the same file is currently used by RSS enclosures and the Vue player. The media handler provides byte-range and cache-validator behavior over stable files. Channel synchronization lists only the current retention window, while favorites may keep older episodes outside that window. FFmpeg and ffprobe are already available in the runtime image.

SponsorBlock provides time intervals on the original YouTube timeline. RSS clients cannot interpret those intervals, while the web player can seek over the original MP3. The two clients therefore need different representations without sacrificing the existing original file.

## Goals / Non-Goals

**Goals:**
- Preserve every original MP3 and generate a stable sponsor-free derivative only when useful.
- Reconcile changed SponsorBlock snapshots without repeating media work for equivalent responses.
- Keep RSS enclosures range-seekable and cacheable as ordinary files.
- Keep frontend progress and controls on the original timeline.
- Make failures non-destructive and retryable.

**Non-Goals:**
- Categories other than `sponsor`, configuration UI, voting, submission, or SponsorBlock user identity.
- Sample-accurate cuts or MP3 re-encoding.
- Automatic SponsorBlock refresh for favorites outside the current channel window.
- Frontend SponsorBlock attribution. A Credits/About placement remains pending follow-up work; this change adds README attribution only.
- Supporting unofficial mirrors or a configurable SponsorBlock base URL.

## Decisions

### D1: Store one atomic snapshot per episode in a separate table

Add a one-to-one `sponsorblock_cache` table keyed by `episode_id` with `ON DELETE CASCADE`. Store normalized segments as JSON together with the full SHA-256 hash, successful check time, active processed filename, processed duration, and optional last error. A successful empty JSON array is distinct from no row, which means never successfully checked.

The application only replaces and consumes segment sets atomically; it does not query individual intervals relationally. A JSON snapshot avoids a segment-row rewrite and reconstruction join on every episode-list query while keeping integration-specific state out of `episodes`.

Alternative considered: columns on `episodes`. Rejected because processing and external-cache state would widen the core episode model and make future SponsorBlock-specific fields harder to evolve.

Alternative considered: one database row per segment. Rejected because no current query addresses segments independently, while ordering and hashing still require reconstructing the complete set.

### D2: Query the official API by plain video id

Use `https://sponsor.ajay.app/api/skipSegments` with `videoID`, category `sponsor`, and action type `skip`. Treat the documented no-segments response as a successful empty snapshot. Network failures, rate limits, malformed responses, and server errors are failures and do not overwrite successful state.

The client follows the repository's existing bounded blocking-HTTP pattern and runs blocking work off the async executor. A timeout and descriptive user agent prevent a channel sync from hanging indefinitely.

Alternative considered: hash-prefix endpoint. Rejected because the requested integration explicitly chooses direct `videoID` lookup and this private installation does not need prefix batching complexity.

### D3: Normalize before hashing or exposing segments

Filter to finite `sponsor`/`skip` intervals, clamp to `[0, original_duration]` when duration is known, discard `end <= start`, sort, and merge overlapping or adjacent intervals. Serialize a canonical payload containing a processing-format version, selected categories, and normalized endpoints, then calculate SHA-256 over those bytes.

Votes, UUIDs, descriptions, and response ordering do not affect the output audio and therefore do not affect the hash. The processing-format version forces deterministic regeneration if normalization or cutting semantics change later. The full hash is stored; a sufficiently long prefix is used in the filename.

### D4: Reconcile only the current synchronization window automatically

After downloads and retention pruning, reconcile stored episodes whose ids appear in the current YouTube selection window. This includes existing recent episodes because `process_episode` currently returns early for them, and includes favorites while they remain recent. Retained favorites outside the window keep their previous snapshot and create no automatic SponsorBlock traffic.

The authenticated manual refresh path bypasses this window rule, giving old favorites an explicit update mechanism.

Alternative considered: scan every stored episode after every sync. Rejected because an unbounded favorite collection would create an unbounded number of external requests.

### D5: Materialize hash-versioned MP3 derivatives with stream copy

Convert sponsor intervals into the complementary intervals to retain. Build an ffconcat description that references the original MP3 repeatedly with `inpoint` and `outpoint`, and invoke FFmpeg with audio stream copy rather than an encoder. MP3 boundaries are therefore aligned to complete codec frames and can differ from SponsorBlock timestamps by roughly one frame; this is an accepted trade-off.

Write to a unique temporary path, require a successful process exit and non-empty output, probe the finished duration with ffprobe, then atomically rename to `{yt_id}.sponsorblock.{hash_prefix}.mp3`. Only after publication succeeds does a database transaction select the new filename. The old derivative is deleted after commit.

Alternative considered: transcode at request time. Rejected because range requests would repeat CPU work and would not map predictably onto generated bytes.

Alternative considered: re-encode the full retained audio. Rejected because sample-level precision is not required and preserving encoded frames avoids generation cost and quality loss.

### D6: Select media in feed generation while the SPA keeps the original URL

Episode reads used by feeds load the SponsorBlock cache state. If `processed_filename` names an existing active file, feed generation uses that filename and its ffprobe-measured duration. Otherwise it uses `{yt_id}.mp3` and the original duration. Channel, legacy, and global feeds apply the same selection helper.

The web player continues to load `/media/{slug}/{yt_id}.mp3`. Episode API serialization adds normalized segments and `sponsorblock_hash`, so its current time, duration, resume threshold, completion, and persisted progress remain original-timeline values.

### D7: Manual refresh is an authenticated synchronous reconciliation

Add an authenticated operation keyed by `yt_id` that runs the same fetch-normalize-reconcile function as synchronization and returns the active snapshot. Stream-copy processing is local and expected to be short enough for this operator-triggered request; sharing the function prevents manual and scheduled behavior from diverging.

Expose this operation through a `Refresh SponsorBlock segments` action in the existing episode-card action menu, including for old favorites. The frontend compares the returned hash with the loaded episode. A changed hash replaces the in-memory intervals without reloading the audio source; an unchanged hash is a no-op. The episode list payload already carries the snapshot, so no request is needed merely to begin playback.

Alternative considered: background job plus polling. Rejected because the project has no general job-status mechanism and the additional state is disproportionate for a manual operation.

### D8: Preserve the last valid state on partial failure

Fetching, derivative generation, publication, and database selection occur as staged operations. A retrieval failure leaves the current row untouched except for optional diagnostic state. A generation or probe failure deletes temporary output and leaves the old processed filename active. A successful empty snapshot switches to the original and retires the old derivative.

There is no time-based expiry: "last valid" remains active until a later successful reconciliation supersedes it. If no valid derivative exists, selection naturally falls back to the original.

### D9: Extend cleanup ownership to derived files

Retention removes the original, active derivative, superseded derivatives, and temporary artifacts for the episode. Orphan cleanup recognizes only the original filenames backed by episode rows and the processed filename referenced by SponsorBlock cache state; stale hash versions and interrupted temporary files are removable. Channel deletion continues removing the whole channel directory.

### D10: Attribute the data source in documentation

Add README credit linking SponsorBlock, `https://sponsor.ajay.app/`, and CC BY-NC-SA 4.0, and state that segment data is transformed into cuts in derived audio. The project must not imply SponsorBlock endorsement. Frontend attribution is deliberately deferred and must remain visible as pending follow-up scope rather than silently dropped.

## Risks / Trade-offs

- **[Risk] Frame-copy cuts are not sample-accurate and may retain or remove about one MP3 frame around a boundary.** -> Accept frame-level precision for sponsor removal and cover boundary behavior with media probes and fixture tests.
- **[Risk] Repeated ffconcat references to one MP3 may expose timestamp, encoder-delay, or compatibility differences across files.** -> Validate generated output with ffprobe and playback tests; never activate output that fails probing.
- **[Risk] An external outage leaves stale SponsorBlock decisions active.** -> Prefer the last known valid derivative over destructive fallback, retry every eligible synchronization, and expose manual refresh.
- **[Risk] API calls add synchronization latency and can trigger rate limiting.** -> Restrict automatic requests to the bounded current window, use timeouts, and process sequentially unless measured evidence supports bounded concurrency.
- **[Risk] Feed enclosure URLs change when the hash changes, causing clients to download a new file.** -> This is intentional cache invalidation; keep the episode GUID stable so it remains the same feed item.
- **[Risk] CC BY-NC-SA 4.0 restricts commercial use and governs adaptations of SponsorBlock data.** -> Document source, license, and transformation in the README; deployments requiring commercial use must obtain compatible permission.

## Migration Plan

1. Apply the additive SponsorBlock cache migration; existing episodes initially have no snapshot and continue serving originals.
2. Deploy the client, reconciliation, API serialization, player skip behavior, and feed selection together. Existing media URLs and progress values remain valid.
3. Subsequent channel synchronizations populate snapshots and derivatives for episodes in each current window. Old favorites remain untouched until manually refreshed.
4. Rollback code can ignore the additive table and hash-versioned files; original MP3s remain present and existing `/media/{slug}/{yt_id}.mp3` URLs continue working. A later cleanup may remove unused derivatives.

## Open Questions

- Frontend attribution placement remains pending. The leading option is a Credits/About entry associated with the authenticated header, but it is outside this change.