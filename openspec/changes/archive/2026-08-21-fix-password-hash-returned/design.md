## Context

`src/models/user.rs` uses `#[derive(Serialize)]` on `User`, whose `hashed_password` is `pub`, so every handler that returns `User` leaks the Argon2 hash. `admin-bootstrap` spec already forbids this; the code needs to catch up.

## Goals / Non-Goals

**Goals:**
- Hash never serialized, anywhere.
- Zero change to the forged response shapes the SPA consumes (beyond dropping the hash).

**Non-Goals:**
- No change to hashing scheme, DB schema, or verification.
- No full DTO refactor of the API unless trivially safe.

## Decisions

- **`#[serde(skip_serializing)]` on `hashed_password`** — minimal, keeps Deserialize (needed to map rows), keeps server-side reads. Alternative (full DTO struct) is more invasive with no current benefit.
- **Audit all serialization sites** for `User` (handlers/users.rs list/read/create/delete, auth flows) and assert the response bodies contain id/name/role/active only.
- **Keep the session payload untouched** — it already carries only id/name/role/active (`SessionUser`), no hash.

## Risks / Trade-offs

- [A future `Serialize`-deriving struct could reintroduce the leak] → Mitigated by the spec requirement and the regression review step in tasks.
- [Frontend expects a `hashed_password` key] → Checked; the SPA types do not reference it; removing the key is safe.

## Migration Plan

Single code change plus a response-body check during verification. No data migration.

## Open Questions

None.