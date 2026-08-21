## Why

`User` derives `Serialize` with `pub hashed_password: String`, so `GET /users/`, single-read, and create/delete responses return the Argon2 password hash of every account to any authenticated caller. The `admin-bootstrap` spec already promises hashes never appear in responses; the implementation violates it. Combined with the CORS flaw (bug #1) a hash could even be exfiltrated cross-origin.

## What Changes

- Exclude `hashed_password` from all serialized `User` responses (`#[serde(skip_serializing)]` or a dedicated response DTO).
- Audit every place a `User` row is serialized (user handlers, session payloads, create/delete returns) and confirm no hash reaches the client.
- Keep the hash readable server-side (verification path `verify_password` must keep working).

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `admin-bootstrap`: Brings the existing "hashes never returned by the API" requirement into actual compliance at the serialization layer.

## Impact

- `src/models/user.rs` (serialization), `src/handlers/users.rs`, anything else returning `User` rows.
- No DB or auth change; verification of hashes still works server-side.
- Regression guard: post-implementation re-analysis against `docs/bug-review-2026-08-21.md`; this fix must not introduce new bugs (e.g. must not break login or any response shape the SPA depends on).