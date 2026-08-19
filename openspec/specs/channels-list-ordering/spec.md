## Purpose

Defines how the channel list is ordered in the Vue 3 SPA and the channel API payload field that supports it. The list defaults to most-recent activity first, and ordering lives in the frontend so additional sort keys can be added without API changes.

## Requirements

### Requirement: Channel API responses expose the last episode date

Every channel returned by `GET /api/1.0/channels/` SHALL include a `last_date` field equal to the channel's most recent episode publication date (`MAX(episodes.published_at)`). A channel with no episodes SHALL return `last_date` as `null`. The remaining channel fields SHALL be unchanged.

#### Scenario: List response includes the latest episode date
- **WHEN** a client requests the channel list
- **THEN** each channel object includes `last_date` holding the publication date of its newest episode

#### Scenario: Channel without episodes has a null last date
- **WHEN** the channel list includes a channel that has no episodes
- **THEN** that channel's `last_date` is `null`

### Requirement: Frontend orders the channel list by last episode

The SPA SHALL render the channels view ordered by `last_date` descending (most recent episode first). Channels whose `last_date` is missing or `null` SHALL be rendered after all channels that have a `last_date`. The ordering SHALL be computed in the frontend so additional sort keys (e.g. title, id) can be added without API changes.

#### Scenario: Newest activity sorts first
- **WHEN** the channels view loads a list where channel A's latest episode is newer than channel B's
- **THEN** channel A is rendered before channel B

#### Scenario: Channels without episodes sort last
- **WHEN** the channels view loads a list containing a channel with no `last_date`
- **THEN** that channel is rendered after every channel that has a `last_date`, regardless of creation date
