## ADDED Requirements

### Requirement: Episodes route displays a channel header

The `/:channelId` route SHALL display a header containing the viewed channel's title and a back arrow to the channel list. The header SHALL be part of the episodes page content (below the shared app header).

#### Scenario: Header appears on the episodes page
- **WHEN** the user navigates to `/app/{channelId}`
- **THEN** the episodes page shows the channel title and a left arrow linking to `/`
