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