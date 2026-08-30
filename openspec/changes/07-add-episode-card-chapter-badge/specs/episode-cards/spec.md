## ADDED Requirements

### Requirement: Card shows a has-chapters indicator
Each `EpisodeCard` SHALL render a small, purely informational icon indicator when its episode has stored chapters. The indicator SHALL NOT be rendered, and no layout space SHALL be reserved for it, when the episode has no stored chapters. The indicator SHALL have no click/tap behavior in this requirement (informational only).

#### Scenario: Episode with chapters shows the indicator
- **WHEN** an episode card is rendered for an episode that has stored chapters
- **THEN** the has-chapters indicator is visible on the card

#### Scenario: Episode without chapters shows no indicator
- **WHEN** an episode card is rendered for an episode with no stored chapters
- **THEN** no has-chapters indicator is rendered and no layout space is reserved for it

#### Scenario: Indicator is present across card presentations
- **WHEN** an episode with stored chapters is rendered in the default, compact, or playlist presentation
- **THEN** the has-chapters indicator is visible in each presentation consistent with that presentation's layout
