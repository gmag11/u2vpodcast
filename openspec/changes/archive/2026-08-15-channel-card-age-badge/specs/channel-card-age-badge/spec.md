## ADDED Requirements

### Requirement: Last episode age badge on channel cards

Each channel card SHALL display a badge in its top-right corner showing the age of the channel's last episode, derived from the channel's `last_date`. The badge SHALL use truncated (non-fractional) units: days `Nd`, weeks `Nw`, months `Nm`, years `Ny`. A partial unit SHALL be truncated down (e.g. a week and a half shows `1w`).

#### Scenario: Badge shows truncated age in days
- **WHEN** a channel's last episode is less than 7 days old
- **THEN** the card shows a badge with the number of days and `d` (e.g. `2d`)

#### Scenario: Badge shows truncated age in weeks
- **WHEN** a channel's last episode is 7 to 29 days old
- **THEN** the card shows a badge with the truncated number of weeks and `w` (e.g. `3w`, and 1.5 weeks shows `1w`)

#### Scenario: Badge shows truncated age in months
- **WHEN** a channel's last episode is 30 to 364 days old
- **THEN** the card shows a badge with the truncated number of months and `m` (e.g. `6m`)

#### Scenario: Badge shows truncated age in years
- **WHEN** a channel's last episode is 365 or more days old
- **THEN** the card shows a badge with the truncated number of years and `y` (e.g. `3y`)

### Requirement: No badge without a last episode date

A channel card whose channel has no `last_date` SHALL NOT render an age badge.

#### Scenario: Channel without episodes shows no badge
- **WHEN** a channel card's channel has a `null` (or missing) `last_date`
- **THEN** the card renders no age badge
