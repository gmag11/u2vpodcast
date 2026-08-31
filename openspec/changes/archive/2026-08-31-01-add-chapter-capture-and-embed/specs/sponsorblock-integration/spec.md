## MODIFIED Requirements

### Requirement: Processed MP3 files are generated only for changed non-empty rejected intervals
When SponsorBlock is enabled and a successful normalized snapshot contains effective rejected intervals whose processing hash differs from the active processing hash, the system SHALL create a derived MP3 named `{yt_id}.sponsorblock.{processing-hash-prefix}.mp3`. The system SHALL preserve the original `{yt_id}.mp3`, copy complete MP3 frames without re-encoding, concatenate the retained intervals, publish the result atomically only after successful completion, and measure the resulting media duration. When the episode has stored chapters, the system SHALL additionally translate each chapter's start and end time from the original timeline onto the derived file's retained-intervals timeline, dropping any chapter whose translated start and end collapse to the same instant, and SHALL embed the translated chapters into the derived MP3 as part of the same generation step, without a separate re-encode or extra pass. An unchanged processing hash SHALL NOT trigger media processing even when non-rejected segment data changed. When SponsorBlock is disabled, no derived media SHALL be generated, selected, replaced, or removed, and no chapter translation SHALL occur.

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

#### Scenario: Chapters fall entirely in retained audio
- **WHEN** a derived MP3 is generated for an episode whose stored chapters all fall within retained intervals
- **THEN** the derived MP3's embedded chapters have the same titles and count as the original chapters, with start and end times shifted to account only for removed audio preceding them

#### Scenario: A chapter is fully contained in a rejected interval
- **WHEN** a stored chapter's start and end both fall within a single rejected interval
- **THEN** that chapter is omitted from the derived MP3's embedded chapters

#### Scenario: A chapter boundary falls inside a rejected interval
- **WHEN** a stored chapter's start or end falls strictly inside a rejected interval rather than at a retained boundary
- **THEN** that boundary is snapped forward to the next retained instant when computing the embedded chapter's time

#### Scenario: Episode has no stored chapters
- **WHEN** a derived MP3 is generated for an episode with no stored chapters
- **THEN** the derived MP3 is produced exactly as before, with no embedded chapters

### Requirement: Retrieval and processing failures preserve the last valid media
SponsorBlock retrieval failures and media-processing failures SHALL NOT replace a successful stored snapshot or active processed file. If no processed file has ever been published, the original MP3 SHALL remain the served representation. A later synchronization or manual refresh SHALL retry the operation. A failure while translating or embedding chapters SHALL be treated as a media-processing failure under this requirement.

#### Scenario: SponsorBlock is unavailable after prior processing
- **WHEN** retrieval fails for an episode with an active processed MP3
- **THEN** the previous snapshot and processed MP3 remain active

#### Scenario: SponsorBlock is unavailable on first check
- **WHEN** retrieval fails for an episode that has no successful SponsorBlock snapshot
- **THEN** the original MP3 remains available and the episode remains eligible for retry

#### Scenario: FFmpeg fails for a changed snapshot
- **WHEN** a changed non-empty snapshot is retrieved but derived MP3 generation fails
- **THEN** no partial file is published and the previously active representation remains selected

#### Scenario: Sponsor segments cover the complete episode
- **WHEN** a normalized snapshot leaves no original-audio interval to retain
- **THEN** the system rejects the snapshot as a processing failure and preserves the previously active representation without generating replacement silence

#### Scenario: Chapter embedding fails after a successful trim
- **WHEN** the concatenated trim succeeds but writing translated chapter metadata into the derived MP3 fails
- **THEN** no partial derived file is published and the previously active representation remains selected
