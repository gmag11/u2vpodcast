## ADDED Requirements

### Requirement: Episodes page has a manual refresh control

The episodes screen SHALL provide a control to refresh the channel being viewed. The control SHALL trigger the channel update endpoint, show loading feedback while the request runs, and surface a notification with the outcome.

#### Scenario: Refreshing episodes from the page
- **WHEN** the user opens a channel's episodes page and activates the refresh control
- **THEN** the SPA calls `POST /api/1.0/channels/{slug}/update/` for the viewed channel and shows loading + a result notification

### Requirement: New channels start updating immediately

When the SPA creates a channel, the backend SHALL begin updating that channel immediately (per the `manual-channel-refresh` capability); the SPA SHALL reflect this by notifying the user that the channel is being processed.

#### Scenario: Channel creation starts processing
- **WHEN** the user creates a channel via the Add dialog and the backend accepts it
- **THEN** the SPA shows a success notification indicating the channel was created and is being updated
