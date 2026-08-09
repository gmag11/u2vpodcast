## Context

See `proposal.md - Why` for the motivation. The relevant existing state for the design:

- `actix-session` with `CookieSessionStore` is already wired in `main.rs:184-202`; sessions carry `user_id`, `user_name`, `user_role`, `user_active` keys (see `src/utils/mod.rs:4-7`).
- `User::get_by_name` (`src/models/user.rs:242`) and `User::check_password` (`src/models/user.rs:128`) already do exactly the credential lookup needed for Basic Auth.
- `from_session` (`src/models/user.rs:41`) already extracts a `SessionUser` from a session for the JSON API path.
- `CustomResponse::new` (`src/models/response.rs:59`) builds the standard `{status, status_code, message, user, data}` body that both `CResponse::ok` and the `Error::error_response` handler already emit, so a `401` body can reuse the same shape.
- `af::Files::new("/media", "./audios")` (`main.rs:228`) serves static files and exposes `.wrap(...)` because `actix-files` `Files` is an `HttpServiceFactory` that accepts middleware wrapping.
- The current routing tree (`src/handlers/mod.rs:22-59`) wraps everything under `""` with a `web::redirect("/", "/app/")` plus `web_feed`, then `web::scope("/api")` containing `web::scope("/1.0")` with all handlers flat. The line `//.wrap(Authentication)` at `mod.rs:51` is already a placeholder for exactly this change.
- `frontend/src/routes/+page.ts` and `frontend/src/routes/[id]/+page.ts` already redirect to `/app/login?next=...` whenever the API response has `user == null`. They will keep working if rejection bodies preserve `user: null`.

## Goals / Non-Goals

**Goals:**
- A single credential pair (admin from `config.yml`) protects the JSON API, the RSS feed, and `/media/**`.
- JSON API stays behind the existing session cookie so the SvelteKit flow is untouched.
- RSS feed and media use HTTP Basic Auth because podcast clients cannot do form login + cookie storage.
- Rejection bodies keep the existing `CustomResponse` shape so the SPA's `+page.ts` redirect path keeps working unchanged.
- Zero DB migrations; reuses `users` and the existing `argon2` hashing path.

**Non-Goals:**
- Multi-user accounts, role differentiation, or per-channel credentials. The admin account stays the only one.
- HTML-level protection of the SPA shell (`/app/**`). The SPA keeps being served as static files; protection rests on the API rejecting unauthenticated data requests. Hardening the static shell is a separate concern.
- Token-in-URL fallback for podcast clients that do not support HTTP Basic Auth. Flagged as a risk; would be a separate change if needed.
- Caching of verified Basic Auth credentials across requests. Every feed/media request re-verifies via argon2 (see Risks for the CPU note).

## Decisions

### Decision 1: Two middlewares, not one

Use **two distinct middleware functions**:

- `require_session` — wraps an actix `wrap_fn` that calls `from_session(req)` and lets the request through on `Ok`, else short-circuits with a `401` whose body is `CustomResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized", empty_session, None)`. Applied only to an inner scope of `/api/1.0` that contains the protected endpoints.
- `basic_auth` — wraps an `actix-web-httpauth::extractors::basic::BasicAuth` extractor and a `wrap_fn` that pulls `Data<AppState>`, resolves `BasicAuth.user_id()` / `.password()`, calls `User::get_by_name(&pool, user)` + `user.check_password(pass).await`, and short-circuits with `401` + `WWW-Authenticate: Basic realm="u2vpodcast"` on any failure. Applied to the `web_feed` scope and to `/media/**`.

**Why not one**: JSON API needs a *cookie* session (extracted via `Session::extract`). Feeds/media need the `Authorization` header (extracted via `BasicAuth`). Trying to fold both into one middleware means the middleware needs to detect which surface it's on, which is more code than two small middlewares.

**Alternative considered**: A single `Authentication` enum middleware that branches on request path. Rejected: the branch lives in `handlers/mod.rs` via the scope tree, which is clearer and matches how actix scopes are meant to be composed.

### Decision 2: `require_session` returns a `CustomResponse` body, not an empty 401

Rather than returning a bare `HttpResponse::Unauthorized()` with no body, the session guard reuses `CustomResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized", session, None)` (see `response.rs:59`) so the response body has `user: null`, `data: null`, `status: false`, `status_code: 401`. This shape is what the SPA `+page.ts` loaders expect.

**Why**: avoids touching the SvelteKit code. Changing the response shape to something the SPA didn't expect would break `+page.ts` and `+page.ts [id]`, requiring frontend changes that the proposal explicitly excludes.

**Alternative considered**: Return `401` empty and patch `+page.ts` to handle `ans.status === 401` explicitly. Rejected because it expands the blast radius to the frontend for no functional gain.

### Decision 3: `basic_auth` returns `401` with `WWW-Authenticate`, never `403`

On any credential failure (unknown user, inactive user, wrong password, missing header), `basic_auth` responds with `401 Unauthorized` and `WWW-Authenticate: Basic realm="u2vpodcast"`. It never returns `403`.

**Why**: the `WWW-Authenticate` challenge is what triggers credential prompts in podcast clients. Returning `403` would cause some clients to silently give up with no opportunity to retry. Treating all failure modes uniformly also avoids leaking which usernames exist.

**Alternative considered**: Distinguish unknown user from wrong password in logs and return codes. Rejected: username enumeration is not worth the security cost for a single-admin system.

### Decision 4: Feed and media verification resolved against the `users` table on every request

The `basic_auth` middleware queries `User::get_by_name(&pool, cred.user_id())` on every request to `/channels/{id}/feed.xml` and `/media/**`, and runs `verify_password` (argon2) every time. No in-memory cache of the admin's hash.

**Why**:
- The admin is a single row; the SQLite pool is `max_connections(2)` (see `main.rs:126`) and queries are point lookups — negligible per request.
- The startup routine (`main.rs:142-147`) wipes and recreates the admin on every boot, so caching risks serving a stale hash if `config.yml` changes between restarts without a redeploy.
- Avoids introducing a new in-memory secret that would have to live in `AppState` and be rotated on hypothetical future admin-edit endpoints.

**Trade-off**: argon2 default parameters (`Argon2::default()`) cost ~50-200 ms per verification. For an admin-only, low-frequency instance (one user subscribing once, then a client fetching the feed periodically), this is fine. If a podcast client does aggressive parallel enclosure downloads, it can spin up several argon2 verifications at once; the worker pool has 2 workers, so it queues behind them. Mitigation if observed as a problem: in-memory cache of only the admin's hash in `AppState`, refreshed on startup. This is **out of scope** for this change; flagged as an open question only.

**Alternative considered**: Cache the admin hash in `AppState` at startup and compare against that, skipping the argon2 round-trip to the DB. Rejected for now in favor of reuse; revisit if the CPU cost is felt.

### Decision 5: Scope tree restructured to keep anonymous endpoints outside the guard

The change reorganizes `handlers/mod.rs`'s scope tree as follows, so `require_session` is applied only to the protected inner scope:

```
""  (root scope)
├─ web::redirect("/", "/app/")                                 (unchanged)
├─ web::scope("")
│    └─ web::resource("/channels/{channel_id}/feed.xml")
│          .route(web::get().to(feed::get_feed))
│          .wrap(basic_auth)                                   ★ NEW
│
└─ web::scope("/api")
     └─ web::scope("/1.0")
          ├─ POST /login/         ─┐
          ├─ GET  /logout/         ─┤
          ├─ GET  /status/         ─┤   ANONYMOUS, no wrap
          ├─ GET  /session/        ─┘
          │
          └─ web::scope("")                                       ★ NEW inner scope
               .wrap(require_session)
               ├─ channels::{
               │     read, read_with_pagination,
               │     create, update, delete
               │  }
               ├─ episodes::read_with_pagination
               └─ config::get_config
```

**Why**: The anonymous endpoints must stay reachable for anyone to log in. Wrapping the whole `/api/1.0` scope would deadlock the login flow. Inner-scope wrapping is the idiomatic actix-web pattern and matches the existing broken-out `//.wrap(Authentication)` comment.

### Decision 6: `/media/**` wrapped via an intermediate scope, not via `af::Files::new().wrap()`

Although `actix-files::Files` exposes `.wrap()`, the cleanest and most portable pattern across actix versions is to mount it inside a guarded scope:

```rust
cfg.service(
    web::scope("/media")
        .wrap(basic_auth)
        .service(af::Files::new("", "./audios"))
);
```

This requires moving `/media` out of the top-level `App::new().service(af::Files::new("/media", ...))` in `main.rs` and into `config_services` in `handlers/mod.rs`. The route is identical from the outside.

**Why**:
- Acts on a confirmed-compatible composition path (a scope wrapping `Files`), avoiding a spike to confirm `Files::wrap()` on the actix 4.5.1 pinned in `Cargo.toml`.
- Keeps all guarded surfaces declared in one place (`handlers::config_services`), which matches where `web_feed` already lives, improving locality and reviewability.

**Alternative considered**: `af::Files::new("/media", "./audios").wrap(basic_auth)` directly in `main.rs`. Rejected to avoid the actix-version compatibility spike and to keep all route/middleware declarations together.

### Decision 7: Add `actix-web-httpauth = "0.8"` for Basic Auth parsing

**Why**:
- Provides the `BasicAuth` extractor that handles parsing `Authorization: Basic`, base64-decoding, and credential extraction. Writing this by hand invites subtle bugs (whitespace, padding, charset).
- Well-maintained and aligned with actix-web 4.

**Alternative considered**: Hand-roll parsing of `req.headers().get(header::AUTHORIZATION)`. Rejected; the crate is small and avoids the parser.

## Risks / Trade-offs

- **[Risk] Some podcast clients don't support HTTP Basic Auth for feeds**, or don't propagate Basic credentials to `<enclosure>` URLs. → Mitigation: before applying, smoke-test with the operator's chosen client against a Basic-protected feed; if it fails, escalate to a token-in-URL approach, which is out of scope here.
- **[Risk] argon2 verification on every media request is CPU-heavy** and parallel enclosure downloads can tie up the 2-worker pool briefly. → Mitigation: acceptable for a single-operator instance; if it becomes a problem, cache the admin hash in `AppState` at startup (deferred — see Open Questions).
- **[Risk] Basic Auth transmits base64 credentials on every request**; without TLS they are recoverable. → Mitigation: the existing `docker-compose.caddy.yml` setup provides TLS in production; `config.yml` already sets `production: true` with `cookie_secure(true)`. The change does not weaken TLS requirements and is safe to ship only behind TLS.
- **[Risk] Wiping users on every restart invalidates any active sessions** imposed by `User::delete_all` + `User::default`. → Mitigation: pre-existing behavior; not introduced by this change. The change does not make it worse, and Basic Auth clients re-authenticate transparently on restart.
- **[Trade-off] `/app/**` static shell remains public.** A determined attacker can load the SPA HTML and assets, but cannot fetch any data without a session. Acceptable in line with the proposal's non-goal.
- **[Trade-off] Parallel Docker layer cache cold after adding `actix-web-httpauth`.** Both `Dockerfile` and `Dockerfile.arm64` (via `build-arm64.sh`) will re-fetch the new crate. Build time increases modestly once. Not user-facing.

## Migration Plan

1. **Spike**: Subscribe the operator's target podcast client to a temporary feed guarded by Basic Auth (can be simulated with a 5-line Python server) and verify the client downloads the feed and an enclosure. If it fails, pause and design the token-URL fallback before applying this change to production.
2. Add `actix-web-httpauth` to `Cargo.toml`; `cargo build` to confirm the dependency resolves and compiles against actix-web 4.5.1.
3. Implement `src/utils/middleware.rs` with `require_session` and `basic_auth`.
4. Restructure `src/handlers/mod.rs` per Decision 5 and move `/media/**` to a guarded scope per Decision 6.
5. Manual sanity: `curl /api/1.0/channels/` returns `401` (no cookie), `/channels/1/feed.xml` returns `401` + `WWW-Authenticate` (no credentials), `/media/1/x.mp3` returns `401` + `WWW-Authenticate`, and each becomes `200` with the correct credentials/cookie.
6. Subscribe the podcast client to the protected feed and confirm a fresh download works end-to-end.
7. Deploy. Operators of any existing podcast client subscriptions must reconfigure them with `admin` / `<admin_password>` (one-time, communicated via release notes).

**Rollback**: Revert the `Cargo.toml`, `src/handlers/mod.rs`, `src/main.rs`, `src/utils/mod.rs`, and `src/utils/middleware.rs` changes. No DB migration to revert. Cookie sessions created before the rollback remain valid.

## Open Questions

- If argon2 verification cost is observed under the operator's real podcast client (parallel enclosure downloads), should we cache the admin hash in `AppState` at startup? Deferrable: the system runs correctly without the cache; only a measurable CPU problem forces the answer. If confirmed, it would be a follow-up change confined to `AppState` + a single preload call in `main.rs`, without touching the spec or the scope tree.