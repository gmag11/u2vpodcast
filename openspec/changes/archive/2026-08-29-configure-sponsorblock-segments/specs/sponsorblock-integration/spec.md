## ADDED Requirements

### Requirement: Rejected SponsorBlock categories are configurable
The system SHALL accept a `sponsorblock_enabled` boolean master switch and a `sponsorblock_rejected_categories` list in `config.yml`. When omitted, SponsorBlock SHALL default to disabled and `sponsor` SHALL default to the only rejected category. When `sponsorblock_enabled` is false, the system SHALL omit all SponsorBlock retrieval, reconciliation, processing, API exposure, manual refresh, playback, marker, and processed-feed-selection behavior regardless of the category list. When enabled, the category list SHALL contain zero or more supported identifiers: `sponsor`, `selfpromo`, `interaction`, `intro`, `outro`, `preview`, `music_offtopic`, and `filler`; duplicates SHALL have no additional effect, and an unsupported identifier SHALL prevent startup with a clear configuration error.

#### Scenario: Both parameters are omitted
- **WHEN** `sponsorblock_enabled` and `sponsorblock_rejected_categories` are absent from `config.yml`
- **THEN** SponsorBlock is disabled and the unused rejected-category selection defaults to `sponsor`

#### Scenario: SponsorBlock is disabled
- **WHEN** `sponsorblock_enabled` is false and the rejected-category list is non-empty
- **THEN** no SponsorBlock functionality runs and the category list has no effect

#### Scenario: Parameter is omitted
- **WHEN** SponsorBlock is enabled and `sponsorblock_rejected_categories` is absent from `config.yml`
- **THEN** only `sponsor` segments are rejected

#### Scenario: Multiple categories are selected
- **WHEN** the parameter contains `sponsor`, `selfpromo`, and `interaction`
- **THEN** segments in all three categories are rejected and segments in the other supported categories remain playable

#### Scenario: No categories are selected
- **WHEN** the parameter is an empty list
- **THEN** no SponsorBlock segments are rejected while available segments remain visible to clients

#### Scenario: Unsupported category is configured
- **WHEN** the parameter contains an identifier outside the supported set
- **THEN** application startup fails with an error that identifies the invalid SponsorBlock category

## MODIFIED Requirements

### Requirement: Sponsor segments are retrieved by YouTube video id
When SponsorBlock is enabled, the system SHALL query the official SponsorBlock API by an episode's `yt_id` and SHALL request `skip` segments for every supported category, independently of which categories are configured for rejection. A successful response with no matching segments, including SponsorBlock's no-segments response, SHALL be treated as an authoritative empty snapshot rather than an error. Entries with unsupported categories or action types other than `skip` SHALL be excluded. When SponsorBlock is disabled, the system SHALL NOT issue SponsorBlock requests.

#### Scenario: Sponsor segments are available
- **WHEN** SponsorBlock returns `skip` segments in supported categories for an episode's `yt_id`
- **THEN** the system accepts every supported segment for normalization regardless of its rejection configuration

#### Scenario: Video has no sponsor segments
- **WHEN** SponsorBlock reports that no matching supported `skip` segments exist for an episode's `yt_id`
- **THEN** the system stores a successful empty snapshot and selects the original MP3 for serving

#### Scenario: Other categories or action types are returned
- **WHEN** a SponsorBlock response contains an unsupported category or an action type other than `skip`
- **THEN** the system excludes that entry from the persisted active segment set

#### Scenario: Retrieval is disabled
- **WHEN** channel synchronization or an episode operation runs while SponsorBlock is disabled
- **THEN** the system makes no request to the SponsorBlock service

### Requirement: SponsorBlock snapshots are reconciled during synchronization
When SponsorBlock is enabled, after channel download and retention processing, the system SHALL refresh SponsorBlock data for every stored episode whose `yt_id` belongs to the channel's current synchronization window. This SHALL include favorite episodes still in that window. Favorite episodes retained outside the current window SHALL keep their last successful snapshot and SHALL NOT be refreshed automatically. When SponsorBlock is disabled, synchronization SHALL bypass reconciliation and leave stored SponsorBlock state and derived files unchanged.

#### Scenario: Existing recent episode is synchronized again
- **WHEN** SponsorBlock is enabled and an already-downloaded episode remains in the channel's current synchronization window
- **THEN** the system requests its SponsorBlock snapshot again during that synchronization

#### Scenario: Recent favorite is synchronized
- **WHEN** SponsorBlock is enabled and a favorite episode belongs to the current synchronization window
- **THEN** the system refreshes its SponsorBlock snapshot like any other recent episode

#### Scenario: Old favorite is outside the synchronization window
- **WHEN** a favorite episode is retained locally but its `yt_id` is not in the current synchronization window
- **THEN** automatic synchronization leaves its stored SponsorBlock snapshot unchanged

#### Scenario: Synchronization runs while disabled
- **WHEN** SponsorBlock is disabled and a channel synchronization completes its download and retention work
- **THEN** SponsorBlock reconciliation is not invoked and existing SponsorBlock cache and derived files remain unchanged

### Requirement: Segment snapshots are normalized and content-addressed
The system SHALL preserve each supported segment's category, clamp valid endpoints to the original episode duration, discard invalid or empty intervals, and order the resulting segments deterministically. Overlapping or adjacent rejected intervals SHALL be merged for playback and media processing even when they belong to different selected categories. The system SHALL derive a deterministic snapshot hash from all normalized categorized segments and a deterministic processing hash from the effective rejected intervals, selected categories, and processing-format version. SponsorBlock metadata that changes neither the visible snapshot nor resulting cuts SHALL NOT affect either hash.

#### Scenario: Equivalent responses have the same hash
- **WHEN** two SponsorBlock responses describe the same categorized intervals in different orders or with different votes, UUIDs, or other non-behavioral metadata
- **THEN** normalization produces the same snapshot hash and processing hash

#### Scenario: Overlapping intervals are returned
- **WHEN** rejected segments from two selected categories overlap or touch
- **THEN** the effective rejected interval set contains one interval covering their union while the descriptive snapshot retains both categorized segments

#### Scenario: Playable segment changes
- **WHEN** a refreshed snapshot changes an interval in a category that is not selected for rejection
- **THEN** the snapshot hash changes and the processing hash remains unchanged

#### Scenario: Effective cut points change
- **WHEN** a refreshed snapshot changes an effective rejected start or end point
- **THEN** both the resulting snapshot hash and processing hash reflect the new state

### Requirement: SponsorBlock state is stored independently from episodes
The system SHALL persist at most one active SponsorBlock snapshot per episode in an independently managed record linked to that episode. The record SHALL distinguish a successful empty snapshot from an episode that has never been checked, and SHALL retain all normalized categorized segments, the snapshot hash, the processing hash and rejected-category selection used for active derived media, the last successful check time, selected processed filename, and processed duration when applicable. Removing an episode SHALL remove its SponsorBlock state. A legacy segment without category metadata SHALL be interpreted as a rejected `sponsor` segment until the episode is refreshed.

#### Scenario: Successful snapshot is persisted
- **WHEN** SponsorBlock retrieval succeeds for a stored episode
- **THEN** the system atomically persists every normalized supported segment and the reconciliation metadata for that episode

#### Scenario: Legacy snapshot is read
- **WHEN** stored segment data contains only start and end values from the previous schema
- **THEN** clients continue to receive it as a rejected `sponsor` segment without losing the prior active media selection

#### Scenario: Episode is deleted
- **WHEN** retention or channel deletion removes an episode
- **THEN** its SponsorBlock state is removed and no orphan relationship remains

### Requirement: Processed MP3 files are generated only for changed non-empty rejected intervals
When SponsorBlock is enabled and a successful normalized snapshot contains effective rejected intervals whose processing hash differs from the active processing hash, the system SHALL create a derived MP3 named `{yt_id}.sponsorblock.{processing-hash-prefix}.mp3`. The system SHALL preserve the original `{yt_id}.mp3`, copy complete MP3 frames without re-encoding, concatenate the retained intervals, publish the result atomically only after successful completion, and measure the resulting media duration. An unchanged processing hash SHALL NOT trigger media processing even when non-rejected segment data changed. When SponsorBlock is disabled, no derived media SHALL be generated, selected, replaced, or removed.

#### Scenario: First non-empty rejected interval set is processed
- **WHEN** an episode has an original MP3 and receives its first non-empty effective rejected interval set
- **THEN** a processing-hash-versioned MP3 is created without replacing the original and becomes the active processed representation

#### Scenario: Only playable segment data changes
- **WHEN** synchronization changes non-rejected segments but leaves the processing hash unchanged
- **THEN** the existing processed MP3 remains active and FFmpeg is not invoked

#### Scenario: Rejected interval set changes
- **WHEN** synchronization obtains a different non-empty processing hash
- **THEN** the system publishes a new processing-hash-versioned MP3 before removing the superseded derived file

#### Scenario: Rejected interval set becomes empty
- **WHEN** a successful refresh leaves no effective rejected intervals
- **THEN** the original MP3 becomes active and the superseded processed MP3 is removed while non-rejected segments remain in the snapshot

#### Scenario: Existing processed media is disabled
- **WHEN** SponsorBlock is disabled for an episode that has an active derived file
- **THEN** the system leaves the derived file and cache state intact but does not select or modify that file

### Requirement: Episode APIs expose SponsorBlock state and allow refresh
When SponsorBlock is enabled, episode payloads SHALL include every normalized supported SponsorBlock segment with its category and whether it is rejected by the active configuration, plus the snapshot hash when available. An authenticated episode refresh operation SHALL retrieve and reconcile the latest SponsorBlock snapshot for that episode regardless of whether it is outside the automatic synchronization window, and SHALL return the resulting active snapshot. A refresh failure SHALL report failure without discarding the last valid snapshot. When SponsorBlock is disabled, episode payloads SHALL expose no SponsorBlock segments or hash, and the refresh operation SHALL report that SponsorBlock is disabled without contacting the service or changing stored state.

#### Scenario: Episode list contains SponsorBlock data
- **WHEN** an authenticated client loads episodes after a successful SponsorBlock check
- **THEN** each checked episode includes all normalized supported segments with `start`, `end`, `category`, and `rejected` values plus its snapshot hash

#### Scenario: Only non-rejected data changes during manual refresh
- **WHEN** an authenticated refresh changes a playable segment but leaves rejected intervals unchanged
- **THEN** the returned snapshot and hash expose the new segment without replacing the active derived media

#### Scenario: Old favorite is refreshed manually
- **WHEN** an authenticated user refreshes SponsorBlock data for a favorite outside the automatic synchronization window
- **THEN** the system retrieves, reconciles, and returns its latest active snapshot

#### Scenario: Manual refresh fails
- **WHEN** the external request or derived media processing fails during manual refresh
- **THEN** the operation reports failure and the prior active snapshot and media remain unchanged

#### Scenario: Episode APIs are used while disabled
- **WHEN** SponsorBlock is disabled and a client loads episodes or requests a SponsorBlock refresh
- **THEN** episode data contains no SponsorBlock snapshot, refresh reports the disabled state, and stored SponsorBlock data is unchanged

### Requirement: SponsorBlock data use is attributed in project documentation
The project README SHALL credit SponsorBlock as the data source, link to `https://sponsor.ajay.app/`, link to CC BY-NC-SA 4.0, and state that configured matching segments are transformed to remove those portions from derived audio. The README and distributed `config.yml` SHALL document `sponsorblock_enabled`, its `false` default and complete bypass semantics, plus `sponsorblock_rejected_categories`, its `sponsor` default, every supported value, empty-list behavior, and the enabled-state distinction between rejection and display. Frontend attribution SHALL remain pending and is not required by this change.

#### Scenario: Operator reviews SponsorBlock configuration
- **WHEN** an operator reads the README or the comments next to the SponsorBlock parameters in `config.yml`
- **THEN** the default-off bypass behavior, category default, supported identifiers, rejection behavior, and enabled-state timeline display behavior are described

#### Scenario: Operator reviews SponsorBlock licensing
- **WHEN** an operator reads the project README
- **THEN** the SponsorBlock source, license, and configured transformation notice are present