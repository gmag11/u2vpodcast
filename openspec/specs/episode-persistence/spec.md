# episode-persistence

## Purpose

Defines correct persistence semantics for the `episodes` table: creation and updates SHALL affect exactly the intended rows, with no self-join side effects.

## Requirements

### Requirement: Updating an episode affects exactly one row

`Episode::update` SHALL update a single episode targetted by its `id` via a plain single-table `UPDATE` (no `FROM` clause against the same table). The statement SHALL return exactly the updated row.

#### Scenario: Updating an existing episode returns one row
- **WHEN** an episode with an existing `id` is saved with modified fields in a database containing several episodes
- **THEN** the update succeeds, the returned row has the given `id` and the new field values, and exactly one row is modified

#### Scenario: The worker create path is unaffected
- **WHEN** a new episode is persisted through the download worker
- **THEN** creation still inserts one row and returns it, as before


### Requirement: Progress can be updated per episode

The episode model SHALL support updating both the playback position and the listened mark (with its timestamp) in a single write, returning the updated episode.

#### Scenario: Position-only update
- **WHEN** progress is saved mid-episode with a position and listened false
- **THEN** the row's `position_seconds` updates and `listen`/`listened_at` stay as before

#### Scenario: Completion update
- **WHEN** progress is saved with the listened flag true
- **THEN** `listen` becomes true, `listened_at` is set, and `position_seconds` stores the final position

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
