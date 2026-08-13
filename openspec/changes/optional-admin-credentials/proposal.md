## Why

Today the service unconditionally wipes the `users` table and reseeds the admin account from `config.yml` on every startup (`main.rs` calls `User::delete_all` then `User::default`). Because `admin_username` and `admin_password` are mandatory config fields, an operator who rotates the admin password by editing the database directly loses that change on the next restart, and the deployment is forced to keep plaintext credentials in `config.yml` even when it already holds the real credentials in the database.

## What Changes

- Make `admin_username` and `admin_password` optional in `config.yml` (both absent, or both empty) for the database-bootstrap decision.
- When the credentials are not both present in config (one missing, one empty, or both absent), **ignore them**: do **not** touch the `users` table at startup — no `delete_all`, no `default` admin creation. Login, feeds, and media authenticate against whatever active row already exists in the database.
- When both credentials are present, keep the existing behavior unchanged: delete all users and reseed the admin from config on every boot.
- Guarantee the database never stores the password in plaintext: in seeded mode the argon2 hash of the config password is stored (never the raw password), and in stored mode only pre-existing hashed rows are used.
- Add a startup log line stating which provisioning mode was used (seeded-from-config vs. kept-existing-database-user).
- Keep `with_authentication` semantics unchanged.

## Capabilities

### New Capabilities
- `admin-bootstrap`: Defines how the admin account is provisioned at startup from either `config.yml` credentials or an existing `users` row, and when the `users` table must be left untouched.

### Modified Capabilities
- `route-protection`: The credential store for the JSON API session login, RSS Basic Auth, and media Basic Auth may now be either the admin reseeded from `config.yml` or a pre-existing active row in the `users` table; the requirement text referencing "the admin account seeded from `config.yml`" needs to allow the stored-credentials path.

## Impact

- `src/models/config.rs`: `admin_username` / `admin_password` become `Option<String>`.
- `src/main.rs`: startup bootstrap logic gated on presence of both credentials.
- `src/handlers/login.rs`, `src/utils/middleware.rs`: unchanged — they already resolve credentials via `User::get_by_name`, which works for either provisioning mode.
- `config.yml` / `README.md`: document that omitting both credentials uses the stored database user.
- No DB schema change. No dependency change.
