# scalable-channel-listing

## Purpose

Defines the bounded, count-window channel sync: the candidate window is the `max` most recent videos in the channel's listing order (never depending on the last-downloaded date), the flat listing is capped to `max + margin` entries, upcoming/live/future-dated videos are excluded, `first` acts as a hard floor, and full episode metadata is obtained per processed video from its download run.

## Requirements

### Requirement: The sync window is the most recent `max` videos

The channel sync SHALL target the `max` most recent videos as its candidate window, regardless of what was previously downloaded. Already-stored videos within the window SHALL be kept untouched; missing ones SHALL be downloaded. The window SHALL NOT depend on the last-downloaded date, so raising `max` automatically adds the older missing videos.

#### Scenario: First backfill stores the N most recent missing videos
- **WHEN** a channel has never been synced, with `first` 3 years ago and `max=20`
- **THEN** the worker selects the 20 most recent videos, skips the ones already stored, and downloads the missing ones (audio + full metadata)

#### Scenario: Raising max recovers older missing episodes
- **WHEN** `max` is raised from 20 to 30 on an already-synced channel
- **THEN** the 21st–30th most recent videos enter the window and the missing ones are downloaded, even though they are older than the previous window

#### Scenario: Unchanged window does no work
- **WHEN** all videos within the current window are already stored
- **THEN** the sync completes without downloads

### Requirement: The window is the newest `max` videos in listing order

The candidate window SHALL be the `max` most recent videos **in the order the channel listing presents them** (the `/videos` tab sorts by publish date, newest first, and the flat listing preserves that order). Publish dates SHALL be used to enforce exclusion rules and the `first` floor — not to re-sort the window. The flat listing SHALL be bounded to `max + margin` entries so the per-cycle cost does not grow with channel age.

#### Scenario: Top-N selection follows the channel listing order
- **WHEN** a channel lists more than `max` videos
- **THEN** the chosen candidates are the first `max` entries in the listing (the `max` most recently published), and older videos are not detail-fetched

#### Scenario: Floor stops the scan early
- **WHEN** walking newest-first, a candidate's date is older than `first`
- **THEN** the walk stops (the remaining entries are older still) and no older candidate is selected

#### Scenario: Shorter catalog covers the whole channel
- **WHEN** a channel has fewer than `max` videos within the floor
- **THEN** all of them are candidates and the listing stops when exhausted

#### Scenario: Listing cost is bounded
- **WHEN** a channel has thousands of videos
- **THEN** the listing requests at most `max + margin` entries (flat), independent of channel size

### Requirement: Upcoming, live and future-dated videos are excluded

Videos that are upcoming (`live_status` is `is_upcoming`), currently live (`is_live`), or whose parsed date is in the future SHALL be excluded from candidates; they SHALL not displace real episodes and SHALL not be downloaded before they become available. Entries without a parseable date SHALL be kept in their listing position (never reordered or ranked by a fabricated date), with their real date resolved and validated at the download step.

#### Scenario: Upcoming premiere is not downloaded
- **WHEN** a candidate is flagged as upcoming
- **THEN** it is excluded from the window and not downloaded

#### Scenario: Future-dated entry is excluded
- **WHEN** a candidate's parsed date is in the future (beyond a small clock-skew tolerance)
- **THEN** it is excluded from the window

#### Scenario: Undated entry keeps its listing position
- **WHEN** a candidate has no parseable date
- **THEN** it keeps its position in the listing (no fabricated date, no reordering); if selected, the download step resolves and validates its real date against the floor

### Requirement: `first` acts as a hard floor

The sync SHALL never select or download videos published before `first`; it SHALL act as a floor, not as a "last downloaded" marker. A candidate whose real date (resolved during download) turns out to predate `first` SHALL be discarded together with its downloaded file, and SHALL NOT be stored.

#### Scenario: Floor excludes older candidates
- **WHEN** `first` is set to a date 3 years ago
- **THEN** no video older than that date is selected or stored

#### Scenario: Undated candidate validated against the floor
- **WHEN** an undated candidate resolves (at download) to a date before `first`
- **THEN** it is discarded with its file and not stored

### Requirement: Full episode metadata is obtained per processed video

Full metadata (description, duration, thumbnail, exact upload date) SHALL be obtained from the download run itself (`--print-json`), under the single YouTube throttle — no separate extraction pass for the listing.

#### Scenario: Missing in-window video downloads with complete metadata
- **WHEN** a candidate inside the window is not yet stored
- **THEN** one throttled run downloads the audio and returns the info dict from which the episode row is built

#### Scenario: Throttle coverage is preserved
- **WHEN** the worker scans and downloads
- **THEN** the flat listing and each per-video run acquire the shared single-connection slot