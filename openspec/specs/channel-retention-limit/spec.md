# channel-retention-limit

## Purpose

Defines the per-channel retention limit (`max`) that caps how many episodes are kept per channel. The limit must be validated server-side on create and update, and pruning must never delete episodes when the stored value is invalid, protecting existing data from accidental wipes.

## Requirements

### Requirement: Retention limit is validated server-side

The per-channel retention limit (`max`) SHALL be accepted only when `>= 1` on channel creation and on channel update. A request with `max` lower than 1 (zero or negative) SHALL be rejected with a 4xx response and a clear error message, and SHALL NOT update the stored value or trigger any pruning.

#### Scenario: Zero retention limit is rejected
- **WHEN** a client submits a channel create or update with `max: 0`
- **THEN** the request is rejected with a 4xx response and no episode data is deleted, neither immediately nor on later syncs

#### Scenario: Negative retention limit is rejected
- **WHEN** a client submits `max: -5`
- **THEN** the request is rejected with a 4xx response and the channel does not enter a permanently failing sync state

#### Scenario: Valid retention limit is accepted
- **WHEN** a client submits `max: 5`
- **THEN** the request succeeds and pruning keeps the newest 5 episodes per channel as before

### Requirement: Pruning never deletes on invalid retention values

`clean_channel` SHALL refuse to delete episodes or files whenever the channel's `max` value is less than 1 (including the DB default of `-1`), regardless of how the value landed in the database. In those cases the sync SHALL still be able to succeed, leaving existing episodes untouched.

#### Scenario: Channel with a stale invalid max is not wiped
- **WHEN** a channel already in the database has `max` <= 0 (e.g. the `-1` default) and the sync runs
- **THEN** no episode is deleted and the sync does not fail because of pruning

#### Scenario: UI input is clamped to the valid range
- **WHEN** the edit dialog receives a `max` value below 1
- **THEN** the input is clamped to 1 before submission


### Requirement: Pruning evicts only non-favorite episodes in excess of the limit

`clean_channel` SHALL count only non-favorite episodes toward a channel's retention limit `max` and SHALL never delete a favorited episode. Eviction SHALL remove the oldest non-favorite episodes, newest first among non-favorites, only while the number of non-favorite episodes exceeds `max`. A favorited episode SHALL be kept regardless of its age or how many episodes the channel holds; adding a new episode SHALL evict the oldest non-favorite episode only when non-favorites already number `max`. When non-favorites are at or below `max`, no episode at all SHALL be deleted, even if total stored episodes (favorites included) exceed `max`.

#### Scenario: Favorites do not count toward the limit
- **WHEN** a channel with `max` 5 stores 5 non-favorite episodes plus 1 favorite and a new episode is published
- **THEN** the 5 non-favorites sit at the limit so nothing is deleted, and the favorite (even as the oldest stored episode) is kept

#### Scenario: Oldest favorite is never evicted
- **WHEN** a channel with `max` 5 stores 4 non-favorite episodes and a favorite that is older than all of them, and a new episode is published
- **THEN** non-favorites number 5, so nothing is deleted, and the old favorite remains

#### Scenario: Excess non-favorites are evicted oldest first
- **WHEN** a channel with `max` 5 stores 6 non-favorite episodes and a favorite, and a new episode is published
- **THEN** the oldest non-favorite episode is deleted, the other 5 non-favorites remain, and the favorite is untouched

#### Scenario: Favorites survive repeated eviction
- **WHEN** a channel keeps publishing episodes so that non-favorites repeatedly exceed `max`
- **THEN** every eviction removes the oldest non-favorite while all favorite episodes remain stored
