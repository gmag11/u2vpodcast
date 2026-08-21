## ADDED Requirements

### Requirement: Last sync age badge on channel cards

Each channel card SHALL display a non-interactive badge in its bottom-left corner showing the elapsed time since the channel's last sync, derived from the channel's `last_sync_at`. The badge SHALL use truncated (non-fractional) units starting in hours: hours `Nh` while under 24 hours, days `Nd` while under 7 days, weeks `Nw` while under 30 days, months `Nm` while under 365 days, and years `Ny` at 365 days or more. A partial unit SHALL be truncated down (e.g. one and a half hours shows `1h`). The badge SHALL sit in the bottom-left corner and SHALL not overlap or conflict with the existing sync-status dot (top-left) or the last-episode age badge (top-right).

#### Scenario: Badge shows truncated hours
- **WHEN** a channel's last sync was less than 24 hours ago
- **THEN** the card shows a badge with the truncated number of hours and `h` (e.g. `1h`, `5h`)

#### Scenario: Badge shows truncated days
- **WHEN** a channel's last sync was 24 hours to 6 days ago
- **THEN** the card shows a badge with the truncated number of days and `d` (e.g. `2d`)

#### Scenario: Badge shows truncated weeks
- **WHEN** a channel's last sync was 7 to 29 days ago
- **THEN** the card shows a badge with the truncated number of weeks and `w` (e.g. `2w`)

#### Scenario: Badge shows truncated months
- **WHEN** a channel's last sync was 30 to 364 days ago
- **THEN** the card shows a badge with the truncated number of months and `m` (e.g. `3m`)

#### Scenario: Badge shows truncated years
- **WHEN** a channel's last sync was 365 or more days ago
- **THEN** the card shows a badge with the truncated number of years and `y` (e.g. `2y`)

#### Scenario: Badge shows zero hours for a fresh sync
- **WHEN** a channel's last sync was less than one hour ago
- **THEN** the card shows `0h` (consistent with the existing sub-unit truncation for the last-episode age badge)

### Requirement: No badge without a last sync timestamp

A channel card whose channel has no `last_sync_at` SHALL NOT render a last-sync-age badge.

#### Scenario: Never-synced channel shows no badge
- **WHEN** a channel card's channel has a `null` (or missing) `last_sync_at`
- **THEN** the card renders no last-sync-age badge
