## 1. Tolerate a missing `CARGO_MANIFEST_DIR`

- [x] 1.1 Add a `dev_root()` helper in `src/main.rs` that returns `CARGO_MANIFEST_DIR` when set, else the current working directory, and logs which branch was chosen
- [x] 1.2 Replace the DB-path `std::env::var("CARGO_MANIFEST_DIR").unwrap()` site with `dev_root()`, keeping `PathBuf` joins (no `to_str().unwrap()`)
- [x] 1.3 Replace the migrations-path `.unwrap()` site with the same helper

## 2. Use `audios_dir()` for slug migration

- [x] 2.1 Change the `Channel::migrate_slugs` call in `src/main.rs` to pass `audios_dir()` instead of the literal `"/app/audios"`
- [x] 2.2 Remove any dead code/literal left behind

## 3. Verification

- [x] 3.1 `cargo run` resolves the crate-relative DB and migrations (existing behavior preserved)
- [x] 3.2 Run the compiled binary directly from a local directory with `RUST_ENV` unset: it starts and resolves paths from CWD without panicking
- [x] 3.3 With an `audios/<id>` directory and an empty-slug channel present, confirm the slug migration renames `audios/<id>` → `audios/<slug>` locally
- [x] 3.4 Full test suite passes