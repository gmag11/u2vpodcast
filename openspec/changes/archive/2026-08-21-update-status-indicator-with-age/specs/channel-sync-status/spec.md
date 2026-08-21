## ADDED Requirements

### Requirement: Status indicator grouped with sync age in bottom-left

Each channel card SHALL render the sync status indicator dot (green when the last sync succeeded, red when it failed) inside the bottom-left corner, immediately to the left of the last-sync age badge, as one visually contiguous group. The card SHALL NOT render the standalone status dot in its top-left corner. A channel that has never been synced (no `last_sync_at` and no success flag) SHALL show neither the dot nor the badge. A channel with a success flag but no timestamp MAY still show the dot.

#### Scenario: Dot sits left of the age badge
- **WHEN** a channel card has a recorded sync outcome and a `last_sync_at`
- **THEN** the card shows the status dot immediately left of the age badge in the bottom-left corner

#### Scenario: No standalone top-left dot
- **WHEN** a channel card is rendered
- **THEN** no status dot appears in the card's top-left corner

#### Scenario: Never-synced channel shows nothing
- **WHEN** a channel has a null `last_sync_at` and no success flag
- **THEN** the card renders neither the status dot nor the age badge

### Requirement: Shared tooltip on the status group

The status dot and age badge group SHALL expose a single shared tooltip on hover (or focus) covering both elements, with the text `Updated <age> ago. Status: Ok` when the last sync succeeded and `Updated <age> ago. Status: Error` when it failed, where `<age>` is the truncated sync age (e.g. `2h`, `3d`). The tooltip SHALL be the sole hover feedback for both the dot and the badge.

#### Scenario: Tooltip for successful sync
- **WHEN** a user hovers over the status group of a card whose last sync succeeded 2 hours ago
- **THEN** the tooltip reads `Updated 2h ago. Status: Ok`

#### Scenario: Tooltip for failed sync
- **WHEN** a user hovers over the status group of a card whose last sync failed 3 hours ago
- **THEN** the tooltip reads `Updated 3h ago. Status: Error`

#### Scenario: Single tooltip covers dot and badge
- **WHEN** a user hovers either the dot or the age badge
- **THEN** the same single tooltip is shown for both elements
