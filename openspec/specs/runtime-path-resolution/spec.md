# runtime-path-resolution

## Purpose

Defines how runtime paths (SQLite DB, migrations, audio storage) are resolved so the app starts and migrates data correctly in any deployment shape, not only "cargo dev" and "Docker production".

## Requirements

### Requirement: Development paths resolve without panicking outside Cargo

When `RUST_ENV` is not `"production"`, the DB file and migrations directory SHALL resolve to `CARGO_MANIFEST_DIR` when that variable is present; when it is absent the app SHALL fall back to the current working directory and log the chosen path. Startup SHALL NOT panic when `CARGO_MANIFEST_DIR` is unset.

#### Scenario: Direct binary execution in development mode
- **WHEN** the compiled binary is run directly (not through Cargo) with `RUST_ENV` unset and a `u2vpodcast.db` next to it
- **THEN** the app resolves the DB and migrations from the working directory, logs that choice, and starts

#### Scenario: Cargo-run still uses the crate root
- **WHEN** the app is started with `cargo run`
- **THEN** the DB and migrations resolve relative to the crate root as today

### Requirement: Slug migration renames audio directories using the shared audio path

The initial slug backfill SHALL rename channel audio directories under the same `audios_dir()` path the worker and delete handler use. The literal Docker-only path SHALL NOT be used for the migration; when `/app/audios` is absent the local `audios` directory SHALL be used.

#### Scenario: Non-Docker audio directory is renamed
- **WHEN** a local deployment has `audios/<id>` and the channel row has an empty slug
- **THEN** the migration renames `audios/<id>` → `audios/<slug>` using the local path

#### Scenario: Docker behavior unchanged
- **WHEN** the container runs with `/app/audios` present
- **THEN** the migration uses `/app/audios` as before