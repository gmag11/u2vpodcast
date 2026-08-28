## Purpose

Integrates SponsorBlock snapshots with episode synchronization so original audio is preserved, sponsor-free media can be generated deterministically, and clients receive stable fallback behavior when external processing fails.

## ADDED Requirements

### Requirement: Sponsor segments are retrieved by YouTube video id
The system SHALL query the official SponsorBlock API by an episode's `yt_id` and SHALL request only `sponsor` segments whose action type is `skip`. A successful response with no matching segments, including SponsorBlock's no-segments response, SHALL be treated as an authoritative empty snapshot rather than an error.

#### Scenario: Sponsor segments are available
- **WHEN** SponsorBlock returns `skip` segments in the `sponsor` category for an episode's `yt_id`
- **THEN** the system accepts those segments for normalization and persistence

#### Scenario: Video has no sponsor segments
- **WHEN** SponsorBlock reports that no matching segments exist for an episode's `yt_id`
- **THEN** the system stores a successful empty snapshot and selects the original MP3 for serving

#### Scenario: Other categories or action types are returned
- **WHEN** a SponsorBlock response contains a category other than `sponsor` or an action type other than `skip`
- **THEN** the system excludes that entry from the persisted active segment set

### Requirement: SponsorBlock snapshots are reconciled during synchronization
After channel download and retention processing, the system SHALL refresh SponsorBlock data for every stored episode whose `yt_id` belongs to the channel's current synchronization window. This SHALL include favorite episodes still in that window. Favorite episodes retained outside the current window SHALL keep their last successful snapshot and SHALL NOT be refreshed automatically.

#### Scenario: Existing recent episode is synchronized again
- **WHEN** an already-downloaded episode remains in the channel's current synchronization window
- **THEN** the system requests its SponsorBlock snapshot again during that synchronization

#### Scenario: Recent favorite is synchronized
- **WHEN** a favorite episode belongs to the current synchronization window
- **THEN** the system refreshes its SponsorBlock snapshot like any other recent episode

#### Scenario: Old favorite is outside the synchronization window
- **WHEN** a favorite episode is retained locally but its `yt_id` is not in the current synchronization window
- **THEN** automatic synchronization leaves its stored SponsorBlock snapshot unchanged

### Requirement: Segment snapshots are normalized and content-addressed
The system SHALL clamp valid segment endpoints to the original episode duration, discard invalid or empty intervals, order intervals by start time, and merge overlapping or adjacent intervals. It SHALL derive a deterministic SHA-256 hash from the normalized intervals, selected categories, and processing-format version. SponsorBlock metadata that does not change the resulting cuts SHALL NOT affect the hash.

#### Scenario: Equivalent responses have the same hash
- **WHEN** two SponsorBlock responses describe the same effective intervals in different orders or with different votes, UUIDs, or other non-cutting metadata
- **THEN** normalization produces the same segment snapshot and hash

#### Scenario: Overlapping intervals are returned
- **WHEN** SponsorBlock returns sponsor intervals that overlap or touch
- **THEN** the system stores one merged interval covering their union

#### Scenario: Effective cut points change
- **WHEN** a refreshed snapshot changes any normalized start or end point
- **THEN** the resulting content hash differs from the stored hash

### Requirement: SponsorBlock state is stored independently from episodes
The system SHALL persist at most one active SponsorBlock snapshot per episode in an independently managed record linked to that episode. The record SHALL distinguish a successful empty snapshot from an episode that has never been checked, and SHALL retain the normalized segments, full hash, last successful check time, selected processed filename, and processed duration when applicable. Removing an episode SHALL remove its SponsorBlock state.

#### Scenario: Successful snapshot is persisted
- **WHEN** SponsorBlock retrieval succeeds for a stored episode
- **THEN** the system atomically persists that episode's normalized snapshot and reconciliation metadata

#### Scenario: Episode is deleted
- **WHEN** retention or channel deletion removes an episode
- **THEN** its SponsorBlock state is removed and no orphan relationship remains

### Requirement: Processed MP3 files are generated only for changed non-empty snapshots
When a successful normalized snapshot contains sponsor intervals and its hash differs from the active processed hash, the system SHALL create a derived MP3 named `{yt_id}.sponsorblock.{hash-prefix}.mp3`. The system SHALL preserve the original `{yt_id}.mp3`, copy complete MP3 frames without re-encoding, concatenate the retained intervals, publish the result atomically only after successful completion, and measure the resulting media duration. An unchanged hash SHALL NOT trigger media processing.

#### Scenario: First non-empty snapshot is processed
- **WHEN** an episode has an original MP3 and receives its first non-empty normalized snapshot
- **THEN** a hash-versioned processed MP3 is created without replacing the original and becomes the active processed representation

#### Scenario: Snapshot hash is unchanged
- **WHEN** synchronization obtains the same effective hash as the active snapshot
- **THEN** the existing processed MP3 remains active and FFmpeg is not invoked

#### Scenario: Snapshot hash changes
- **WHEN** synchronization obtains a different non-empty effective hash
- **THEN** the system publishes a new hash-versioned processed MP3 before removing the superseded derived file

#### Scenario: Snapshot becomes empty
- **WHEN** SponsorBlock successfully changes an episode from a non-empty snapshot to no sponsor segments
- **THEN** the original MP3 becomes active and the superseded processed MP3 is removed

### Requirement: Retrieval and processing failures preserve the last valid media
SponsorBlock retrieval failures and media-processing failures SHALL NOT replace a successful stored snapshot or active processed file. If no processed file has ever been published, the original MP3 SHALL remain the served representation. A later synchronization or manual refresh SHALL retry the operation.

#### Scenario: SponsorBlock is unavailable after prior processing
- **WHEN** retrieval fails for an episode with an active processed MP3
- **THEN** the previous snapshot and processed MP3 remain active

#### Scenario: SponsorBlock is unavailable on first check
- **WHEN** retrieval fails for an episode that has no successful SponsorBlock snapshot
- **THEN** the original MP3 remains available and the episode remains eligible for retry

#### Scenario: FFmpeg fails for a changed snapshot
- **WHEN** a changed non-empty snapshot is retrieved but derived MP3 generation fails
- **THEN** no partial file is published and the previously active representation remains selected

### Requirement: Episode APIs expose SponsorBlock state and allow refresh
Episode payloads SHALL include the normalized SponsorBlock segments and snapshot hash when available. An authenticated episode refresh operation SHALL retrieve and reconcile the latest SponsorBlock snapshot for that episode regardless of whether it is outside the automatic synchronization window, and SHALL return the resulting active snapshot. A refresh failure SHALL report failure without discarding the last valid snapshot.

#### Scenario: Episode list contains SponsorBlock data
- **WHEN** an authenticated client loads episodes after a successful SponsorBlock check
- **THEN** each checked episode includes its normalized segments and snapshot hash in the episode payload

#### Scenario: Old favorite is refreshed manually
- **WHEN** an authenticated user refreshes SponsorBlock data for a favorite outside the current synchronization window
- **THEN** the system retrieves, reconciles, and returns its latest active snapshot

#### Scenario: Manual refresh fails
- **WHEN** the external request or derived media processing fails during manual refresh
- **THEN** the operation reports failure and the prior active snapshot and media remain unchanged

### Requirement: Derived media follows episode file lifecycle
Retention cleanup and orphan cleanup SHALL recognize the active hash-versioned SponsorBlock file as belonging to its episode. Removing an episode or channel SHALL remove its original and derived MP3 files, while superseded temporary or derived files SHALL be eligible for cleanup.

#### Scenario: Retention evicts a processed episode
- **WHEN** retention removes an episode that has original and processed MP3 files
- **THEN** both representations and the episode's SponsorBlock state are removed

#### Scenario: Active derived file is scanned for orphans
- **WHEN** orphan cleanup examines a hash-versioned processed MP3 referenced by active SponsorBlock state
- **THEN** the file is retained

### Requirement: SponsorBlock data use is attributed in project documentation
The project README SHALL credit SponsorBlock as the data source, link to `https://sponsor.ajay.app/`, link to CC BY-NC-SA 4.0, and state that segment data is transformed to remove matching portions from derived audio. Frontend attribution SHALL remain pending and is not required by this change.

#### Scenario: Operator reviews SponsorBlock licensing
- **WHEN** an operator reads the project README
- **THEN** the SponsorBlock source, license, and transformation notice are present