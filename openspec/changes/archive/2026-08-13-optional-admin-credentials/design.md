## Context

The service seeds a single admin account on every startup. `src/main.rs` unconditionally calls `User::delete_all(&pool)` followed by `User::default(&pool, &config.admin_username, &config.admin_password)`. Both `admin_username` and `admin_password` are mandatory `String` fields in `src/models/config.rs` with no `#[serde(default)]`, so `serde_yaml::from_str` fails if either is missing and the process panics. The only consumer of these two config fields is the bootstrap block in `main.rs`; the login handler (`src/handlers/login.rs`) and the Basic Auth middleware (`src/utils/middleware.rs`) already resolve credentials against the `users` table via `User::get_by_name` + `verify_password`, so they are agnostic to how the row got there.

## Goals / Non-Goals

**Goals:**
- Allow an operator to omit both admin credentials from `config.yml` and have the service authenticate against the existing `users` table rows without modifying them.
- Keep the current seed-on-startup behavior when at least one credential is present (backward compatible).
- Emit a clear startup log line identifying the active provisioning mode.

**Non-Goals:**
- Introducing a second credential store, multi-user management, or a UI for managing users.
- Changing `with_authentication` semantics or the `401` response contract.
- Handling a partial config (only one of the two fields set) as a stored-mode trigger — that stays in seeded mode to preserve today's behavior.

## Decisions

### D1: Model the two config fields as `Option<String>`

Change `admin_username` and `admin_password` in `Config` from `String` to `Option<String>`. Serde maps an absent YAML key to `None` and an empty string to `Some("")`; both cases are treated as "absent" for the bootstrap decision via a helper such as `fn admin_credentials_present(&self) -> bool` that returns `self.admin_username.as_deref().filter(|s| !s.is_empty()).is_some() && self.admin_password.as_deref().filter(|s| !s.is_empty()).is_some()`.

- **Rationale:** minimal API change; single field type change, no new config struct. `Option<String>` keeps serde defaults implicit (`None` for missing keys).
- **Alternative considered:** `#[serde(default)]` with empty-string defaults on `String` — rejected because it loses the distinction between "missing" and "present but empty", and requires sentinel semantics.
- **Alternative considered:** a `#[serde(default)]` on a custom enum `AdminConfig::{Seed, Stored}` — more explicit but heavier than needed for a two-field gate.

### D2: Gate the bootstrap block in `main.rs` on presence of both credentials

Replace the unconditional `delete_all` + `default` block with:

```
if config.admin_credentials_present() {
    User::delete_all(&pool).await.expect(...);
    User::default(&pool, username, password).await.expect(...);
    info!("Admin reseeded from config.yml (seeded mode)");
} else {
    info!("admin_username/admin_password not both set; ignoring config credentials and keeping existing users table");
}
```

The `expect` calls are retained in seeded mode so a seed failure still aborts startup as today; in stored mode no DB write happens and nothing can fail. `admin_credentials_present()` returns `true` only when both values are non-empty, so a partial config (one field set) falls into stored mode and is ignored.

- **Rationale:** keeps the guard at the single call site, zero impact on handlers or middleware.
- **Alternative considered:** moving the decision into `User::default` — rejected: `User` model should not depend on `Config`, and the delete step must stay gated too.

### D3: No migration of hashed passwords

Stored mode uses whatever rows are already in `users`. No re-hash, no password reset. The argon2 hashes stored by `User::new` are already compatible with `verify_password`, so existing rows are immediately usable. Password hashing (`token_utils::hash_password`, argon2) stays the single write path in seeded mode — the raw config password is never persisted, only its hash.

### D4: Keep `with_authentication` and 401 semantics untouched

`with_authentication` still only toggles Basic Auth on feed/media. In stored mode with an empty `users` table, all authenticated surfaces return `401` (unless the flag is `false`). This matches the spec scenario and requires no middleware change.

## Risks / Trade-offs

- [Operator removes credentials accidentally] → Config-less start with empty DB yields a service nobody can log into. Mitigation: startup log line clearly states stored mode; documentation tells operators to verify a user exists before removing credentials.
- [Partial config (one field set) now silently ignored] → Previously the config crashed on load; now it starts in stored mode and may surprise an operator who expected seeded mode. Mitigation: log line clearly names stored mode; README documents that both fields must be set to enable seeding.
- [Old serde behavior changed from panic to graceful start] → A config missing credentials previously failed fast. Now it starts. Mitigation: the explicit info log line removes the surprise; README documents both modes.

## Migration Plan

1. Deploy the new binary unchanged; existing `config.yml` with both credentials behaves exactly as before (seeded mode).
2. To adopt stored mode: ensure a valid active row exists in `users` (or log in once in seeded mode), then remove both `admin_username` and `admin_password` keys from `config.yml` and restart.
3. Rollback: add the credentials back to `config.yml`; the next restart reseeds the admin, restoring today's behavior.

## Open Questions

- None blocking. Optionally: should a warning be emitted when stored mode is active and `users` is empty? (Currently covered by the existing `401` path and the startup log.)
