# channel-retention-limit

## Purpose

Defines the per-channel retention limit (`max`) that caps how many episodes are kept per channel. The limit must be validated server-side on create and update, and pruning must never delete episodes when the stored value is invalid, protecting existing data from accidental wipes.

## ADDED Requirements

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