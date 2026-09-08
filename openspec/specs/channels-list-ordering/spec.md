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

### Requirement: Frontend orders the channel list by a configurable sort

The SPA SHALL render the channels view ordered by a user-configurable sort. The sort key SHALL default to `last_date` descending (most recent episode first) and SHALL be changeable between `last_date`, `title`, and `id`. The sort direction SHALL be changeable between ascending and descending. Channels whose `last_date` is missing or `null` SHALL be treated as the oldest and sort accordingly: first in ascending order, last in descending order. The ordering SHALL be computed in the frontend; the channel API SHALL NOT change.

#### Scenario: Default sort is most recent episode first
- **WHEN** the channels view loads with the default sort
- **THEN** channels are rendered by `last_date` descending, newest episode first

#### Scenario: Sort alphabetically by title
- **WHEN** the user selects the title sort key
- **THEN** channels are rendered in case-insensitive alphabetical order by title

#### Scenario: Sort by id
- **WHEN** the user selects the id sort key
- **THEN** channels are rendered ordered by id

#### Scenario: Direction reverses the selected key
- **WHEN** the user toggles the sort direction to ascending
- **THEN** the sort order for the selected key is reversed

#### Scenario: Channels without episodes sort as oldest
- **WHEN** the sort key is `last_date` and a channel has no `last_date`
- **THEN** that channel is treated as the oldest: rendered first when ascending and last when descending

### Requirement: Channels view exposes sort controls

The channels view SHALL render controls that let the user pick the sort key (`last_date`, `title`, or `id`) and the sort direction (ascending/descending). The controls SHALL reflect the current selection. The selected key and direction SHALL persist across page reloads.

#### Scenario: Controls change the order
- **WHEN** the user changes the sort key or direction through the controls
- **THEN** the rendered channel list re-orders accordingly

#### Scenario: Selection persists across reloads
- **WHEN** the user reloads the channels view
- **THEN** the previously selected sort key and direction are still applied
