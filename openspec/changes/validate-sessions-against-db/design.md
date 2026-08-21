## Context

`from_session` (`src/models/user.rs`) builds a `SessionUser` from cookie claims only. `RequireSession` (`middleware.rs`) calls it and that is all. The `route-protection` spec already requires resolving to an active `users` row. `SessionOrBasicAuth` hits the DB only in its Basic Auth branch. Bootstrap reseed (`main.rs`) deletes/recreates users without invalidating cookies.

## Goals / Non-Goals

**Goals:**
- Every session-protected request is DB-validated.
- Deactivated/deleted/reseeded users rejected immediately.
- Claims (role/active) always current.

**Non-Goals:**
- No session-storage revamp, no server-side session registry, no cookie invalidation list (DB lookup makes those unnecessary).
- No change to cookie signing/lifecycle.

## Decisions

- **Single DB-backed lookup helper** (`User::get_by_id`) used by `from_session` or the middleware, replacing cookie-claim trust. Implementation: middleware fetches the row by `user_id`; on `None` or `active=false` → `401`. Keep cookie claims as the id selector only.
- **Refresh claims each request** from the fetched row; this also makes the future role stage (bug #2) build on fresh data.
- **Reuse for `SessionOrBasicAuth`** session branch so feed/media behave identically.
- **Per-request cost:** one `SELECT` on the primary key per authenticated request; at this scale (single user) it is noise; rejected as a concern.

## Risks / Trade-offs

- [Extra DB round-trip per request] → Bounded by PK lookup; acceptable for correctness; can be revisited only if profiling says otherwise.
- [Bootstrap reseed still valid in config-less mode] → The lookup reads whatever the table holds; reseed replaces rows and lookup naturally rejects stale ids.
- [Tests asserting session behavior may need a seeded user row] → Expected; test fixture already seeds the admin.

## Migration Plan

Code change only. Existing cookies keep working for live users; stale ones start failing immediately (desired).

## Open Questions

None.