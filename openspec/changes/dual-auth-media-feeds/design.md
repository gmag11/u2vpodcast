## Context

Currently `BasicAuthGuard` (in `src/utils/middleware.rs`) protects `/media/**` and the feed routes. It only checks the `Authorization: Basic` header. The `RequireSession` middleware protects the JSON API and only checks the session cookie. See `proposal.md` for motivation and `specs/route-protection/spec.md` for the new requirements.

The existing building blocks:
- `RequireSessionMiddleware.call()` (`middleware.rs:75-96`): calls `from_session(req.get_session())` to resolve a user from the cookie; returns `200` through or `401` with a JSON `CustomResponse`.
- `BasicAuthMiddleware.call()` (`middleware.rs:140-175`): extracts `BasicAuth` from the request header, validates against DB; returns `200` through or `401` with `WWW-Authenticate: Basic realm="u2vpodcast"`.

Both middlewares are already implemented and stable.

## Goals / Non-Goals

**Goals:**
- A single middleware that accepts a request if *either* the session cookie *or* the Basic Auth header resolves to an active user.
- Apply this middleware to `/media/**`, `/channels/{slug}/feed.xml`, and `/{slug}/feed.xml`.
- The `with_authentication` flag still bypasses the guard when `false`.
- Maintain identical `401` response shape per surface: `WWW-Authenticate: Basic` for feeds and media; no change to the SPA's existing `CustomResponse`-based handling on `/api`.

**Non-Goals:**
- Changing the JSON API session guard (`RequireSession`).
- Adding a third auth method (tokens, JWTs, API keys).
- Changing the credential store or the `with_authentication` logic.
- Affecting CORS or rate-limiting.
- Adding session-based auth to any other route.

## Decisions

### Decision 1: Single combined `SessionOrBasicAuth` middleware

Add a new middleware struct `SessionOrBasicAuth` (and its `SessionOrBasicAuthMiddleware` service) that:
1. Checks `with_authentication` from `AppState` — if `false`, passes immediately (identical to the existing `BasicAuthGuard` behavior).
2. Extracts the session via `req.get_session()` and calls `from_session` — if successful, passes.
3. If session check fails, extracts `BasicAuth` from the request header and validates against DB — if successful, passes.
4. If both fail, returns `401` with `WWW-Authenticate: Basic realm="u2vpodcast"`.

**Why**: one middleware with `OR` logic is simpler than chaining two separate guards (which `actix-web` resolves sequentially with `AND` semantics — chained `.wrap()` calls would require *both* to pass, not *either*). The current `BasicAuthGuard` and `RequireSession` can stay in the codebase for the JSON API (which still uses session-only).

**Alternative considered**: wrapping routes with two separate guards. Rejected because `actix-web` applies middleware as a pipeline; wrapping `RequireSession` then `BasicAuthGuard` would require BOTH to pass, not EITHER. The `OR` semantics must live inside a single `Service.call()`.

### Decision 2: Session is checked before Basic Auth

The session cookie is tried first; Basic Auth is only attempted if the session is absent or invalid. The `401` response always includes `WWW-Authenticate: Basic` to maintain compatibility with podcast clients.

**Why**: for the primary use case (SPA audio element), the session cookie is always present and checking it is a cheap `from_session` call that doesn't hit the DB (actix-session stores the session in a cookie-signed store, not a DB call). Basic Auth requires a SQL `User::get_by_name` + argon2 hash comparison, so only paying that cost when the session is absent is more efficient.

### Decision 3: Reuse existing helper functions, don't inline

The new middleware calls the same `from_session` and `User::get_by_name` + `check_password` paths that the existing guards use. The `from_session` function is imported from `crate::models` and the DB access goes through `AppState.pool` (identical to `BasicAuthMiddleware`).

**Why**: no duplicated auth logic, no risk of session or password verification drifting out of sync between the three guards.

### Decision 4: Keep existing `BasicAuthGuard` and `RequireSession` in the codebase

The JSON API still uses `RequireSession` alone. Both existing middlewares remain in `middleware.rs` unchanged. The route configs in `mod.rs` and `feed.rs` are the only files that change — they replace the `BasicAuthGuard` import and usage with `SessionOrBasicAuth`.

**Why**: minimizes blast radius. If a future change needs Basic-Auth-only or session-only on a new route, the original guards are still available.

### Decision 5: No change to `from_session` or `CustomResponse` shape

The `401` response on feed/media routes stays a plain empty body with `WWW-Authenticate: Basic realm="u2vpodcast"` — no `CustomResponse` JSON is emitted, because podcast clients and audio elements don't parse JSON and the `WWW-Authenticate` header is what triggers the native credential prompt. The JSON API session guard continues to return its existing `CustomResponse` shape unchanged.

**Why**: the SPA's `<audio>` element follows redirects; it doesn't parse JSON. Emitting a JSON body on `401` from `/media` would be harmless noise but would require extra code. The `WWW-Authenticate` header is the standard mechanism for HTTP Basic Auth negotiation and is what podcast clients check.

## Risks / Trade-offs

- **[Risk] Session cookie sent to `/media` in cross-origin scenarios.** If the app were hosted on a different domain from the media server, cookies wouldn't be sent. → Mitigation: in this deployment, the SPA, API, feeds, and media are served from the same `actix-web` process on the same origin. Same-origin cookie sending is automatic.
- **[Risk] Dual-auth path could mask a broken session if Basic Auth is also cached by the browser.** A user with both a stale session and cached Basic Auth could silently fall back to Basic Auth. → Mitigation: both methods resolve the same user row; authorization is identical. If the session is stale and Basic Auth works, the request still reaches the resource correctly.
- **[Trade-off] `WWW-Authenticate: Basic` added even when the user has a valid session but wrong Basic Auth header.** A browser with a `404` from a session-authenticated request won't see the header because sessions take priority. If a request has both a good session and bad Basic Auth, the session wins and no prompt appears — which is the desired behavior.

## Migration Plan

1. Add `SessionOrBasicAuth` middleware in `src/utils/middleware.rs`.
2. In `src/handlers/feed.rs`: replace `BasicAuthGuard` import and usage with `SessionOrBasicAuth` on both feed routes.
3. In `src/handlers/mod.rs`: replace `BasicAuthGuard` on the `/media` scope with `SessionOrBasicAuth`.
4. `cargo build` and deploy.
5. Verify: a logged-in browser accessing a feed or audio file receives `200` without a Basic Auth prompt. A podcast client without a session cookie still receives `401` with `WWW-Authenticate` and retries with Basic Auth.

**Rollback**: revert the three files. No DB schema change, no config change.

## Open Questions

None.
