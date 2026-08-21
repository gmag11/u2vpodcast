## ADDED Requirements

### Requirement: Slug uniqueness enforced by the database

The `slug` column SHALL be backed by a UNIQUE index/constraint in the database. Duplicate slug rows SHALL be impossible regardless of request concurrency. The migration SHALL deduplicate any existing rows (appending `-N` suffixes deterministically) before creating the index, and SHALL be reversible.

#### Scenario: Migration fixes existing duplicates
- **WHEN** the migration runs on a database where two channels already share a slug
- **THEN** the duplicate row receives a unique `-N` suffix and the UNIQUE index is created successfully

#### Scenario: Concurrent identical titles yield distinct slugs
- **WHEN** two channels with the same title are created concurrently
- **THEN** both inserts succeed with distinct slugs (e.g. `confesiones_de_gasolinera` and `confesiones_de_gasolinera-2`), one of them via conflict retry

### Requirement: Slug creation handles unique violations by suffix retry

When an insert attempts a slug that already exists (race window), slug generation SHALL detect the unique violation and retry with the next `-N` suffix instead of failing or producing a duplicate.

#### Scenario: Race loser retries with a suffix
- **WHEN** a create request loses the race on its chosen slug
- **THEN** it retries with `-2`, `-3`, … until it finds a free slug and the insert succeeds

### Requirement: Channel deletion removes only its own audio directory

Deleting a channel SHALL remove the audio directory only when it belongs exclusively to that channel. If another channel still references the same directory path (should be impossible once slugs are unique, but the guard must exist), deletion SHALL NOT remove the shared directory's foreign files; it may skip directory removal and log a warning instead of risking foreign data loss.

#### Scenario: Delete removes owned directory
- **WHEN** a normal channel with its own slug directory is deleted
- **THEN** its audio directory and files are removed as today

#### Scenario: Delete never wipes a foreign directory
- **WHEN** a deletion finds a directory path that another remaining channel also uses
- **THEN** the shared directory's files are left in place and a warning is logged rather than `remove_dir_all` destroying them