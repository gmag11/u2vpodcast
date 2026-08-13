## 1. Config model

- [x] 1.1 Change `admin_username` and `admin_password` to `Option<String>` in `src/models/config.rs`
- [x] 1.2 Add `Config::admin_credentials_present()` helper returning `true` only when both values are non-empty

## 2. Startup bootstrap

- [x] 2.1 Gate the `User::delete_all` + `User::default` block in `src/main.rs` behind `config.admin_credentials_present()`
- [x] 2.2 Emit an `info!` log line naming the provisioning mode (seeded vs. stored) in both branches

## 3. Documentation

- [x] 3.1 Document in `README.md` that the `users` table is only touched when both `admin_username` and `admin_password` are set; omitting either keeps the existing database user untouched
- [x] 3.2 Add a commented example in `config.yml` showing the stored-credentials mode
- [x] 3.3 Document in `README.md` that setting only one of the two credentials is ignored (stored mode)

## 4. Verification

- [x] 4.1 Build with `cargo build` and fix any compile errors
- [x] 4.2 Manual test: run with credentials present in `config.yml` and confirm the admin is reseeded and login works
- [x] 4.3 Manual test: remove both credentials from `config.yml`, keep a user in the DB, confirm the table is untouched and login with stored credentials works
- [x] 4.4 Manual test: remove both credentials with an empty `users` table and confirm every authenticated surface returns `401`
- [x] 4.5 Manual test: set only `admin_username` (omit password) in `config.yml`, confirm both credentials are ignored, the `users` table is untouched, and login uses the stored database user
- [x] 4.6 Confirm the `hashed_password` column never holds the plaintext config password (seeded mode stores the argon2 hash via `User::new`)
