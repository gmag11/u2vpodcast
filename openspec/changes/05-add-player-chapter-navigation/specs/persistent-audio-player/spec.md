## ADDED Requirements

### Requirement: Expanded view offers chapter-level previous/next navigation
When the current episode has stored chapters, the expanded "now playing" view SHALL display previous-chapter and next-chapter controls, distinct from the existing episode-level previous/next controls. Activating next-chapter SHALL seek to the start of the chapter immediately after the one containing the current playback position; if the current position is within the last chapter, the control SHALL be disabled. Activating previous-chapter SHALL restart the current chapter (seek to its start) when more than 3 seconds have elapsed since that chapter's start, and SHALL otherwise seek to the start of the preceding chapter; if the current position is within the first chapter and at or before 3 seconds into it, the control SHALL be disabled. Both controls SHALL apply existing rejected-interval skip behavior after seeking when SponsorBlock is enabled. When the current episode has no stored chapters, neither control SHALL be rendered.

#### Scenario: Next-chapter advances to the following chapter
- **WHEN** the user activates next-chapter while playback is within a chapter that has a following chapter
- **THEN** the player seeks to the start of the following chapter

#### Scenario: Next-chapter is disabled in the last chapter
- **WHEN** playback is within the episode's last chapter
- **THEN** the next-chapter control is disabled

#### Scenario: Previous-chapter restarts the current chapter
- **WHEN** the user activates previous-chapter more than 3 seconds after the current chapter's start
- **THEN** the player seeks to the start of the current chapter

#### Scenario: Previous-chapter moves to the preceding chapter
- **WHEN** the user activates previous-chapter within 3 seconds of the current chapter's start and a preceding chapter exists
- **THEN** the player seeks to the start of the preceding chapter

#### Scenario: Previous-chapter is disabled at the episode's first chapter start
- **WHEN** playback is within 3 seconds of the first chapter's start
- **THEN** the previous-chapter control is disabled

#### Scenario: Chapter navigation respects rejected intervals
- **WHEN** a chapter-navigation seek lands on a time inside a segment marked as rejected, and SponsorBlock is enabled
- **THEN** the player applies the existing rejected-interval skip behavior after seeking

#### Scenario: Episode has no stored chapters
- **WHEN** the current episode has no stored chapters
- **THEN** neither chapter-navigation control is rendered
