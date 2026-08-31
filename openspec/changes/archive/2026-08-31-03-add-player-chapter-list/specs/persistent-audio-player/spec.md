## ADDED Requirements

### Requirement: Expanded view lists chapters with jump-to-chapter interaction
When the current episode has stored chapters, the expanded "now playing" view SHALL display a "Chapters" section listing every chapter's title and start time in order. Activating a chapter row SHALL seek playback to that chapter's start time, subject to the existing rejected-interval skip behavior when that time falls inside a segment marked as rejected. The row corresponding to the chapter containing the current playback position SHALL be visually highlighted and SHALL update as playback progresses. When the current episode has no stored chapters, the expanded view SHALL display no Chapters section.

#### Scenario: Chapters section appears for an episode with chapters
- **WHEN** the expanded view is open for an episode that has stored chapters
- **THEN** a Chapters section lists every chapter's title and start time in order

#### Scenario: Activating a chapter row seeks to its start
- **WHEN** the user taps or clicks a chapter row
- **THEN** the shared player seeks to that chapter's start time

#### Scenario: Chapter row seek respects rejected intervals
- **WHEN** the user activates a chapter row whose start time falls inside a segment marked as rejected, and SponsorBlock is enabled
- **THEN** the player applies the existing rejected-interval skip behavior after seeking

#### Scenario: Current chapter is highlighted
- **WHEN** playback position falls within a chapter's time range
- **THEN** that chapter's row is visually highlighted while no other row is

#### Scenario: Highlight follows playback
- **WHEN** playback advances past a chapter boundary
- **THEN** the highlighted row updates to the new current chapter without user interaction

#### Scenario: Episode has no stored chapters
- **WHEN** the current episode has no stored chapters
- **THEN** the expanded view shows no Chapters section
