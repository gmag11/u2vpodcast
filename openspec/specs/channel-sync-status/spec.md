## Purpose

Defines how the system records and exposes the outcome of per-channel sync attempts, and how the Vue 3 SPA surfaces that status to users via a sync status indicator dot on channel cards. The system records a timestamp and success/failure for the most recent sync of each channel, exposes it through the channel API, and renders a green/red indicator on each channel card based on the latest recorded outcome.

## Requirements

### Requirement: Track last sync outcome per channel

The system SHALL record the outcome of the most recent sync attempt for each channel, persisting a timestamp of the last sync attempt and whether it succeeded or failed. A sync attempt covers the full per-channel refresh (fetching new videos and processing new episodes). Success SHALL be recorded when the refresh completes without error, regardless of whether new episodes were found. Any error (HTTP 429/503, cookie/auth failure, download failure, or any other unexpected error) SHALL be recorded as a failure.

#### Scenario: Successful sync records success
- **WHEN** a channel refresh completes without any error, with or without new episodes
- **THEN** the system records the sync as successful with the current timestamp

#### Scenario: Failed sync records failure
- **WHEN** a channel refresh returns an error of any kind (e.g. HTTP 429, 503, cookie/auth failure, download failure)
- **THEN** the system records the sync as failed with the current timestamp

#### Scenario: Sync outcome recorded from both sync paths
- **WHEN** a channel is refreshed by the background worker or by a manual per-channel refresh request
- **THEN** the sync outcome is recorded in both cases

### Requirement: Channel API exposes sync status

The channel API payload SHALL include the last sync timestamp and whether the last sync succeeded. A channel that has never been synced SHALL expose a null timestamp and no success flag (or an explicit "never synced" representation).

#### Scenario: Channel payload includes sync status
- **WHEN** a client fetches the channel list or a single channel
- **THEN** each channel includes its last sync timestamp and last sync success flag

#### Scenario: Never-synced channel has empty sync status
- **WHEN** a client fetches a channel that has never been synced
- **THEN** the channel's last sync timestamp is null and no success flag is set

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

### Requirement: Indicator reflects latest recorded sync

The card indicator SHALL reflect the most recently recorded sync outcome for the channel, as returned by the channel API. After a refresh (background or manual) completes and records its outcome, the next channel list fetch SHALL show the updated indicator.

#### Scenario: Indicator reflects latest recorded outcome
- **WHEN** a channel sync completes and records its outcome, and the channel list is subsequently fetched
- **THEN** the card indicator reflects the latest recorded sync outcome
