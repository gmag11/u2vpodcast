## Why

The deployment runs a single admin account (`role-authorization` spec). The user-management endpoint `DELETE /api/1.0/users/` (`src/handlers/users.rs:98`) deletes any user by id with no guard. Deleting the only (or last active) admin permanently locks everyone out until credentials in `config.yml` are (re)provided or the database is edited by hand. That lockout is unrecoverable in deployments without `admin_username`/`admin_password` seeding.

The `role-authorization` spec defers full role management (and the "prevent deleting the last active admin" guard) to a future stage — that deferral must be honored, but the self-inflicted lockout is a live bug in the current single-admin deployment and is worth fixing now as a minimal safety invariant, not a role system.

## What Changes

- Refuse deletion of the last active administrator: when the target user is an active admin and no other active admin would remain after the deletion, the request fails with a `409 Conflict` (or `400`) and a clear message.
- The check happens in the model layer (`User::delete`) or immediately adjacent in the handler so every call path is covered.
- Existing behavior for all other rows (non-admin target, or another admin remaining) is unchanged.

## Capabilities

### New Capabilities

- `admin-account-protection`: Defines the safety invariant that the last active administrator can never be deleted, independent of the deferred role-management system.

### Modified Capabilities

- `role-authorization` (referenced, not modified): this change instantiates exactly one deferred sub-guarantee now; full role enforcement remains deferred.

## Impact

- `src/models/user.rs` (guard in `delete`), possibly `src/handlers/users.rs`.
- No API schema change; one new documented error response.
- No frontend change required (it already displays error responses from this endpoint).

## Non-Goals

- No per-route role checks; no session-role enforcement.
- No change to `User::default`/`delete_all` seeding path (startup reseed is an explicit operator action and remains allowed).
- No user-creation flow changes.