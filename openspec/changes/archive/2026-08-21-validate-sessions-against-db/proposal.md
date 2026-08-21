## Why

Authorization relies solely on claims baked into the signed session cookie at login (`user_id`, `role`, `active`). Nothing re-checks the `users` table, so a user who is deactivated or deleted keeps full API access until the 7-day TTL expires. Additionally, the startup reseed path (`User::delete_all` + seed) leaves previously issued cookies valid against a users table where the old row no longer exists. The `route-protection` spec already requires resolution to an active user; the middleware does not implement it.

## What Changes

- `RequireSession` (and the session branch of `SessionOrBasicAuth`) SHALL resolve the session's `user_id` against the `users` table and reject the request with `401` when the row is missing or inactive.
- Role/name/active claims may be refreshed from the DB row so the session reflects current state.
- After a reseed, cookies for deleted users are rejected on the next request with no waiting period.
- Feed/media Basic Auth path is unchanged (it already hits the DB).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `route-protection`: Closes the gap between the spec ("resolves to an active user in the `users` table") and the implementation (cookie-only claims).

## Impact

- `src/utils/middleware.rs`, `src/models/user.rs` (`from_session` or a new DB-backed lookup), session handling in `src/handlers/login.rs` (claim refresh).
- New overhead: one indexed single-row `SELECT` per authenticated request (cost negligible on this DB shape).
- Regression guard: re-analysis against `docs/bug-review-2026-08-21.md`; the DB lookup must not introduce new bugs (e.g. must not break the reseed/bootstrap flows or the config-less auth mode).