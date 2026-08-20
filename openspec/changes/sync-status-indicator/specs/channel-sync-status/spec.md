## ADDED Requirements

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

### Requirement: Sync status indicator on channel cards

Each channel card SHALL display an indicator dot in its top-left corner reflecting the last sync outcome: green when the last sync succeeded, red when it failed. A channel that has never been synced SHALL show no indicator (or a neutral/absent state). The indicator is non-interactive and non-blocking.

#### Scenario: Green dot on successful sync
- **WHEN** a channel card's last sync succeeded
- **THEN** the card shows a green dot in its top-left corner

#### Scenario: Red dot on failed sync
- **WHEN** a channel card's last sync failed
- **THEN** the card shows a red dot in its top-left corner

#### Scenario: No dot when never synced
- **WHEN** a channel card's channel has never been synced
- **THEN** the card shows no sync indicator dot

### Requirement: Indicator reflects latest recorded sync

The card indicator SHALL reflect the most recently recorded sync outcome for the channel, as returned by the channel API. After a refresh (background or manual) completes and records its outcome, the next channel list fetch SHALL show the updated indicator.

#### Scenario: Indicator reflects latest recorded outcome
- **WHEN** a channel sync completes and records its outcome, and the channel list is subsequently fetched
- **THEN** the card indicator reflects the latest recorded sync outcome
