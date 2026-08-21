## Context

`src/main.rs` resolves two path families:

1. SQLite DB file + migrations directory. Production: relative to `std::env::current_exe()` parent, `db/` (Docker has a writable `db/` next to the binary). Development: `CARGO_MANIFEST_DIR` joined with the crate root, which only exists when running through Cargo. `CARGO_MANIFEST_DIR` is a Cargo-injected env var; a directly executed binary or a systemd/vertical deployment without it panics on `.unwrap()`.
2. Audio storage. `audios_dir()` (`src/models/config.rs`) already selects `/app/audios` when present else `audios` (relative). `main.rs` ignores this helper and passes the literal `"/app/audios"` to `Channel::migrate_slugs`, so the directory rename half of the slug migration silently no-ops outside Docker (it reports success — the files just never move). Since slugs are what the UI and feed use for `/media/{slug}/...`, downloads after migration keep working, but the mismatch leaves stale `id/` directories behind.

## Goals / Non-Goals

**Goals:**
- A non-Cargo, non-Docker runtime resolves sensible local paths instead of panicking.
- Audio directory renames honor the same `audios_dir()` selection as every other audio access.

**Non-Goals:**
- No new configuration surface for paths (keep `RUST_ENV` as the switch).
- No change to Docker/production layouts.

## Decisions

- **Dev-path resolver:** introduce a small helper, e.g. `fn dev_root() -> PathBuf`, returning `CARGO_MANIFEST_DIR` when present and otherwise the current working directory, logging which branch was taken. Replace both `.unwrap()` sites with it; keep `.to_str()` fallback behavior by joining onto a `PathBuf` and never stringifying until needed. If the resulting DB directory does not exist, `sqlx` creation/migrate still fails loudly, so a wrong fallback cannot silently misdirect — but the crash on startup is gone.
- **Audios path:** call `Channel::migrate_slugs(&pool, audios_dir())` and delete the now-unused literal. This makes post-migration environment behavior consistent with `Channel::delete` (which already uses `audios_dir()` via the handler) and the worker (`audios_dir()` in `worker.rs`).
- **Same helper for both DB and migrations:** both resolve via the same `dev_root()` so they cannot diverge.

## Risks / Trade-offs

- [Falling back to CWD may resolve a different DB than `cargo run`] → No: when run under Cargo the env var is present and wins; outside Cargo, CWD is the only sane anchor and the startup log prints the chosen path.
- [`audios_dir()` returning `audios` relative to CWD in dev] → Matches existing behavior of the worker and delete handler; consistency is the point.

## Migration Plan

1. Add `dev_root()` helper; rewire dev DB and migrations resolution through it.
2. Swap the `migrate_slugs` call to `audios_dir()`.
3. Verify: `cargo run` still uses the crate-relative DB; run the built binary directly from a directory, confirm it starts and resolves paths from CWD; in a local run with an `audios/<id>` dir present, confirm the slug rename executes.

## Open Questions

None.