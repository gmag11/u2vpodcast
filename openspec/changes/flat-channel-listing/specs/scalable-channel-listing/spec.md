# scalable-channel-listing

## ADDED Requirements

### Requirement: Channel listing is flat and does not extract every video

Listing a channel's coverage SHALL be cheap: a flat scan that returns in-window video references from the channel API pages, without fully extracting out-of-window videos (no per-video webpage/JS-challenge/PO-token round trip during the listing itself). The listing SHALL complete in seconds for channels with thousands of videos, bounded by the number of pages, not by the number of videos in the date window.

#### Scenario: Large channel is listed without per-video extraction
- **WHEN** a channel with thousands of videos is scanned for a backfill window
- **THEN** the listing returns the candidate references quickly and transparently (logged per episode, not a silent multi-minute scan)

#### Scenario: Out-of-window videos are not fully extracted by the listing
- **WHEN** the scan encounters videos older than the channel's `first`/`last` boundary
- **THEN** they are excluded from candidates without full metadata extraction

### Requirement: Full episode metadata is deferred to per-video processing

Full metadata (description, duration, thumbnail, exact upload date) SHALL be fetched only for videos that will be stored: in-window and not already present. Each new episode SHALL obtain its metadata from its own download run (one connection), under the single YouTube throttle.

#### Scenario: In-window new video stores complete metadata
- **WHEN** a candidate video inside the window is not yet stored
- **THEN** its full metadata is obtained during its own processing run and the episode row is complete (title, description, duration, thumbnail, upload date)

#### Scenario: Already-stored videos are not reprocessed
- **WHEN** the scan revisits a video already present
- **THEN** no metadata fetch or download is performed for it

#### Scenario: Out-of-window videos never trigger detail fetch
- **WHEN** a flat candidate has a timestamp older than the window
- **THEN** it is skipped before any per-video work (backstop filtering when the flat listing cannot apply `--dateafter` itself)

### Requirement: Date window is honored without extraction cost

The `first`/`last` boundary SHALL be applied to flat entries (via `--dateafter --break-on-reject` where the extractor supports it, and/or by Rust-side timestamp comparison) so the number of per-video detail fetches is proportional to the in-window candidates, not the channel size.

#### Scenario: Backfill window candidates are bounded
- **WHEN** a channel has 4000 videos and the window covers 1000
- **THEN** at most the ~1000 in-window candidates are considered for per-video work, and the rest are skipped by date comparison alone

#### Scenario: Empty window costs only the listing
- **WHEN** no video falls inside the window
- **THEN** the cycle completes with zero per-video detail fetches

### Requirement: Throttle coverage is preserved

The flat listing, any per-video detail fetch, and every download SHALL remain inside the single-connection YouTube throttle (youtube-throttling), so this change never opens parallel connections or bypasses the cooldown.

#### Scenario: Listing and downloads serialize with other YouTube traffic
- **WHEN** the worker scans a channel while other channels or image fetches are active
- **THEN** the listing and each per-video run acquire the shared slot like any other YouTube operation