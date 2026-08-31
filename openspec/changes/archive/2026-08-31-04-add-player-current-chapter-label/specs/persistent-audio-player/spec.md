## ADDED Requirements

### Requirement: Player compositions show the current chapter title
When the current episode has stored chapters, the persistent player's compact and wide compositions and the expanded "now playing" view SHALL display the title of the chapter containing the current playback position as a secondary label near the episode title. The label SHALL update without requiring user interaction as playback crosses chapter boundaries. When the current episode has no stored chapters, or playback position is before the first chapter's start, no chapter label SHALL be shown, and no layout space SHALL be reserved for it. In the compact composition, the label SHALL appear between the episode title and the channel/playback-time line.

#### Scenario: Chapter label shown for an episode with chapters
- **WHEN** any player composition is displayed for an episode with stored chapters and playback is within a chapter's range
- **THEN** that chapter's title is shown as a secondary label near the episode title

#### Scenario: Label updates across chapter boundaries
- **WHEN** playback advances from one chapter into the next
- **THEN** the displayed label updates to the new current chapter's title without user interaction

#### Scenario: Episode has no stored chapters
- **WHEN** the current episode has no stored chapters
- **THEN** no chapter label is shown and no layout space is reserved for it

#### Scenario: Compact composition places the chapter label between existing metadata
- **WHEN** the compact composition (viewport narrower than 640px) is displayed
- **THEN** the current chapter title is rendered between the episode title and the channel/playback-time line
