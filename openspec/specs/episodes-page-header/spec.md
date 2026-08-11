## Purpose

Defines the channel header shown on the episodes page: a left arrow back to the channel list and the title of the channel being viewed.

## Requirements

### Requirement: Episodes page shows the channel title with a back arrow

The episodes screen SHALL render a header at the top of the page content showing the title of the channel being viewed, preceded by a left arrow control. Activating the arrow SHALL navigate to the channel list (`/`).

#### Scenario: Episodes page shows the channel header
- **WHEN** an authenticated user opens `/app/42`
- **THEN** the page renders a header with a left arrow and the title of the channel with id `42`

#### Scenario: Back arrow returns to the channel list
- **WHEN** the user clicks the left arrow in the episodes page header
- **THEN** the router navigates to the channel list route (`/`)

### Requirement: Channel title is resolved for the episodes page

The episodes page SHALL resolve the current channel's title so the header can display it. When the title cannot be resolved (e.g., the channel is not found), the header SHALL show a neutral fallback (e.g., "Episodes") instead of failing or showing an empty title.

#### Scenario: Title resolved from the channel list
- **WHEN** the episodes page loads and the channel list contains a channel matching the route id
- **THEN** the header shows that channel's title

#### Scenario: Channel not found shows a fallback
- **WHEN** the episodes page cannot resolve a matching channel
- **THEN** the header displays a neutral fallback title and the back arrow still navigates to the channel list
