## Why

Runtime path resolution assumes exactly two deployment shapes, and breaks anywhere else:

- In development mode (`RUST_ENV != "production"`) the DB and migrations paths are built from `std::env::var("CARGO_MANIFEST_DIR").unwrap()` (`src/main.rs:138,155`). Running the compiled binary outside `cargo` (or with the var unset) panics at startup instead of resolving a sensible local path.
- The slug migration is called with the hard-coded literals `"/app/audios"` (`src/main.rs:201`) even though a `audios_dir()` helper already exists that falls back to the local `audios/` directory. Outside a Docker deployment the audio directories are never renamed to their slugs because the code looks in `/app/audios` which does not exist.

## What Changes

- Resolve the dev-mode DB and migrations directory through a helper that tolerates `CARGO_MANIFEST_DIR` being absent, falling back to the current working directory, and logs the chosen path.
- Replace the hard-coded `"/app/audios"` in the `migrate_slugs` call with the shared `audios_dir()` helper so slug backfill and directory renames work in any deployment.

## Capabilities

### New Capabilities

- `runtime-path-resolution`: Defines how DB, migrations, and audio paths are resolved at startup and at runtime, independently of the deployment shape.

### Modified Capabilities

(none)

## Impact

- `src/main.rs` (dev-mode path resolution, `migrate_slugs` call site).
- No API contract change; no schema change.
- The production path behavior is unchanged.

## Non-Goals

- No change to the production path layout (`/app/...` stays as-is for Docker).
- No environment-variable-driven path configuration beyond the existing `RUST_ENV` switch.