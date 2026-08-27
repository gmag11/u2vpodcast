# episode-persistence

## Purpose

Defines correct persistence semantics for the `episodes` table: creation and updates SHALL affect exactly the intended rows, with no self-join side effects.

## MODIFIED Requirements

### Requirement: Episode persistence stores playback position and listened time

The episodes table SHALL store, per episode, the last playback position in seconds (defaulting to 0), the timestamp of completion (`listened_at`, nullable), and a favorite flag (defaulting to false). The existing `listen` boolean SHALL represent the played/completed mark. Reads and other updates SHALL preserve these fields, including the favorite flag.

#### Scenario: New episodes store zero progress
- **WHEN** an episode row is created by the download worker
- **THEN** `position_seconds` is 0, `listened_at` is null, `listen` is false, and `favorite` is false

#### Scenario: Existing episodes migrate with zero progress
- **WHEN** the migration runs over a database with existing episodes
- **THEN** all existing rows get `position_seconds` 0, null `listened_at`, and `favorite` false without dropping data

#### Scenario: Favorite survives unrelated updates
- **WHEN** an episode stored with `favorite` true is updated for any other reason (title, progress, listened mark)
- **THEN** the `favorite` flag keeps its stored value