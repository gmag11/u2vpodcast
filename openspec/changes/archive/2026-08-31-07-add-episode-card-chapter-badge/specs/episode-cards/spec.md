## ADDED Requirements

### Requirement: Card shows a has-chapters indicator
Each `EpisodeCard` SHALL render a small, purely informational icon indicator alongside its favorite and playlist icons when its episode has stored chapters. The indicator SHALL expose a localized tooltip on hover or keyboard focus. The indicator SHALL NOT be rendered when the episode has no stored chapters. The mobile playlist presentation SHALL keep a fixed chapter-icon slot so its status icons remain aligned between rows; other presentations SHALL reserve no space for an absent indicator. The indicator SHALL have no click/tap behavior in this requirement (informational only).

#### Scenario: Episode with chapters shows the indicator
- **WHEN** an episode card is rendered for an episode that has stored chapters
- **THEN** the has-chapters indicator is visible on the card

#### Scenario: Episode without chapters shows no indicator
- **WHEN** an episode card is rendered for an episode with no stored chapters
- **THEN** no has-chapters indicator is rendered
- **AND** only the mobile playlist presentation retains an empty status slot to preserve row alignment

#### Scenario: Mobile playlist status icons stay aligned
- **WHEN** mobile playlist cards with and without stored chapters are rendered together
- **THEN** favorite, playlist, and chapter status slots remain in fixed positions across the rows

#### Scenario: Indicator explains its meaning
- **WHEN** a listener hovers over or focuses the has-chapters indicator
- **THEN** a localized tooltip identifies that the episode has chapters

#### Scenario: Indicator is present across card presentations
- **WHEN** an episode with stored chapters is rendered in the default, compact, or playlist presentation
- **THEN** the has-chapters indicator is visible alongside the favorite and playlist icons in each presentation consistent with that presentation's layout
