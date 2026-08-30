## ADDED Requirements

### Requirement: Timeline displays chapter markers on the original media timeline
When an episode has stored chapters, the persistent player's progress track SHALL render a marker at each chapter's original start time, using the episode's stored, untranslated chapter times against the original media duration — consistent with the shared player always operating on the original timeline. This applies to the wide composition's interactive scrubber, the expanded view's scrubber, and the compact composition's read-only track. Chapter markers SHALL be visually distinct from SponsorBlock segment markers and from the playback-progress fill. On an interactive scrubber (wide or expanded), activating a chapter marker SHALL seek playback to that chapter's start time, subject to the existing rejected-interval skip behavior when that time falls inside a segment marked as rejected. The compact composition's read-only track SHALL render chapter markers without accepting seek interaction, consistent with its existing no-seek behavior. An episode with no stored chapters SHALL render no chapter markers.

#### Scenario: Chapter markers appear on the wide scrubber
- **WHEN** the wide composition is displayed for an episode that has stored chapters
- **THEN** a marker appears at each chapter's original start time along the scrubber, distinct from any SponsorBlock markers present

#### Scenario: Chapter markers appear on the expanded scrubber
- **WHEN** the expanded "now playing" view is open for an episode that has stored chapters
- **THEN** the same chapter markers appear on its scrubber at the same relative positions as the wide composition

#### Scenario: Chapter markers appear on the compact track
- **WHEN** the compact composition is displayed for an episode that has stored chapters
- **THEN** chapter markers appear on the read-only progress track and do not accept seek interaction

#### Scenario: Activating a chapter marker seeks to its start
- **WHEN** the user clicks or taps a chapter marker on the wide or expanded scrubber
- **THEN** the shared player seeks to that chapter's original start time

#### Scenario: Chapter marker falls inside a rejected SponsorBlock interval
- **WHEN** the user activates a chapter marker whose start time falls inside a segment marked as rejected, and SponsorBlock is enabled
- **THEN** the player applies the existing rejected-interval skip behavior after seeking, landing at the end of the complete overlapping rejected interval

#### Scenario: Episode has no stored chapters
- **WHEN** the current episode has no stored chapters
- **THEN** no chapter markers are rendered on any composition's track

#### Scenario: Chapter and SponsorBlock markers are visually distinguishable
- **WHEN** a progress track shows both chapter markers and SponsorBlock segment markers
- **THEN** the two marker types use distinct visual treatments so neither is mistaken for the other
