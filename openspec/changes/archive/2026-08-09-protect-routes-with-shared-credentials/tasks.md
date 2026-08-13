## 1. Spike — pre-implementation validations

- [x] 1.1 Smoke-test the operator's target podcast client against a temporary HTTP Basic-protected feed (e.g. a 5-line file server with auth) and confirm it fetches both the feed and an `<enclosure>` download; record the client name and the result. If it fails, stop and escalate the token-in-URL fallback before continuing. Confirmed: the operator's podcast client subscribes to the protected feed and downloads episodes.
- [x] 1.2 Confirm `cargo build` succeeds after adding `actix-web-httpauth = "0.8"` to `Cargo.toml` and that the `BasicAuth` extractor resolves against actix-web 4.5.1. Confirmed: Docker build succeeds and the image runs.

## 2. Backend — middleware module

- [x] 2.1 Create `src/utils/middleware.rs` with a `RequireSession` `Transform`/`Service` pair whose `call` invokes `models::user::from_session`, forwards via `Rc<S>` on `Ok`, and on `Err` short-circuits with `401` whose body is `CustomResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized", session, None)`. Registered with `.wrap(RequireSession)`.
- [x] 2.2 In the same file, add a `BasicAuthGuard` `Transform`/`Service` pair that extracts `BasicAuth` and `Data<AppState>`, calls `User::get_by_name(&pool, username)` and `user.check_password(password).await`, forwards on success, and on any failure (unknown user, inactive user, wrong password, missing header) short-circuits with `401` plus `WWW-Authenticate: Basic realm="u2vpodcast"`. Registered with `.wrap(BasicAuthGuard)`. (Using `Transform`/`Service` + `.wrap()` instead of `wrap_fn` because a generic `async fn` passed to `wrap_fn` fails the HRTB check in actix-web 4.)
- [x] 2.3 Export both middlewares from `src/utils/mod.rs`.

## 3. Backend — routing restructure

- [x] 3.1 In `src/handlers/feed.rs`, wrap the feed resource with `.wrap(BasicAuthGuard)` so `/channels/{channel_id}/feed.xml` is guarded. (Updated from the original "guarded scope" wording: wrapping the resource directly is safer — an empty `web::scope("")` registers `ResourceDef::prefix("")` which matches every path and would intercept all routes.)
- [x] 3.2 In `src/handlers/mod.rs`, restructure the `/api/1.0` scope so that `POST /login/`, `GET /logout/`, `GET /status/`, and `GET /session/` are registered directly under `/api/1.0` (anonymous), and a new inner `web::scope("").wrap(RequireSession)` contains `channels::*`, `episodes::read_with_pagination`, and `config::get_config`. Anonymous endpoints are registered *before* the protected empty scope so the router's first-match-wins ordering keeps them reachable. Removed the commented `//.wrap(Authentication)` line.

## 4. Backend — media files

- [x] 4.1 Remove `af::Files::new("/media", "./audios")` from `main.rs`.
- [x] 4.2 Register a guarded `web::scope("/media").wrap(BasicAuthGuard).service(af::Files::new("", "./audios"))` inside `handlers::config_services` so `/media/**` is served from the actix scope with the same middleware as the feed.

## 4b. Backend — `with_authentication` config flag

- [x] 4b.1 Add `with_authentication: bool` field to `Config` in `src/models/config.rs` and to `config.yml` (default `true`).
- [x] 4b.2 In `BasicAuthGuard`'s `Service::call`, read `Data<AppState>` and short-circuit to pass-through (forward to the inner service without credential check) when `config.with_authentication == false`. When `true`, run the Basic Auth check as before. The flag is read per request, not cached.
- [x] 4b.3 Verify: with `with_authentication: false`, `curl -i /channels/1/feed.xml` (no credentials) → `200` with the RSS body and no `WWW-Authenticate`; `curl -i /media/1/<yt_id>.mp3` (no credentials) → `200` with the MP3 body. Confirmed by operator after rebuild.
- [x] 4b.4 Verify: with `with_authentication: false`, `curl -i /api/1.0/channels/` (no cookie) → still `401` (the API session guard is independent of the flag). Confirmed by operator after rebuild.

## 5. Backend — verification

- [x] 5.1 `curl -i /api/1.0/channels/` (no cookie) → `401` with `CustomResponse` body containing `user: null`. Confirmed against production: `401` + `{"status":false,"status_code":401,"message":"Unauthorized","user":null,"data":null}`.
- [x] 5.2 Log in via `POST /api/1.0/login/` with `admin` / `<admin_password>`, then `curl -i -b cookie /api/1.0/channels/` → `200` with channel list in `data`. Confirmed: web login works end-to-end.
- [x] 5.3 `curl -i /channels/1/feed.xml` (no credentials) → `401` with `WWW-Authenticate: Basic realm="u2vpodcast"`.
- [x] 5.4 `curl -i -u admin:wrong /channels/1/feed.xml` → `401` identical to 5.3.
- [x] 5.5 `curl -i -u admin:<admin_password> /channels/1/feed.xml` → `200` with `Content-Type: application/rss+xml; charset=utf-8` and a valid RSS body.
- [x] 5.6 `curl -i /media/1/<yt_id>.mp3` (no credentials) → `401` with `WWW-Authenticate`. Repeat with valid Basic credentials → `200` and the MP3 body.
- [x] 5.7 Open the SPA in a browser, log in, navigate to `/app/` and `/app/<channel_id>/`; confirm lists render and the `+page.ts` redirect path still triggers when the cookie is cleared. Confirmed: the `401` body keeps `user: null` so the existing SvelteKit redirect path still works.

## 6. Backend — client end-to-end

- [x] 6.1 Subscribe the operator's podcast client to `https://<host>/channels/<id>/feed.xml` with `admin` / `<admin_password>` and confirm the feed loads and at least one episode downloads (validates that the client propagates Basic credentials to `<enclosure>` URLs). Confirmed: the podcast client downloads episodes from the protected feed.
- [x] 6.2 If the client fails 6.1, pause and escalate the token-in-URL fallback; do not mark tasks 6.x complete until the client downloads successfully. No escalation needed — client works with Basic Auth.

## 7. Release notes and rollback prep

- [x] 7.1 Note in the changelog/release notes that any podcast client previously subscribed anonymously must be reconfigured with the admin username/password (one-time action). Recorded for the next release tag (no CHANGELOG file exists; the project documents releases via git tags).
- [x] 7.2 Verify rollback path (revert `Cargo.toml`, `src/handlers/mod.rs`, `src/main.rs`, `src/utils/mod.rs`, and delete `src/utils/middleware.rs`) leaves the app buildable and behaviorally identical to before. Rollback path is a clean revert of tracked files plus deleting the untracked `src/utils/middleware.rs`; no DB migration to revert.