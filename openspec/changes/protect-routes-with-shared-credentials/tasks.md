## 1. Spike — pre-implementation validations

- [ ] 1.1 Smoke-test the operator's target podcast client against a temporary HTTP Basic-protected feed (e.g. a 5-line file server with auth) and confirm it fetches both the feed and an `<enclosure>` download; record the client name and the result. If it fails, stop and escalate the token-in-URL fallback before continuing.
- [ ] 1.2 Confirm `cargo build` succeeds after adding `actix-web-httpauth = "0.8"` to `Cargo.toml` and that the `BasicAuth` extractor resolves against actix-web 4.5.1.

## 2. Backend — middleware module

- [ ] 2.1 Create `src/utils/middleware.rs` with a `require_session` `wrap_fn` that calls `models::user::from_session`, lets the request through on `Ok`, and on `Err` short-circuits with `401` whose body is `CustomResponse::new(StatusCode::UNAUTHORIZED, "Unauthorized", session, None)` (reusing `response.rs:59`).
- [ ] 2.2 In the same file, add `basic_auth` as a `wrap_fn` that extracts `Data<AppState>` and `BasicAuth`, calls `User::get_by_name(&pool, username)` and `user.check_password(password).await`, lets the request through on success, and on any failure (unknown user, inactive user, wrong password, missing header) short-circuits with `401` plus `WWW-Authenticate: Basic realm="u2vpodcast"`. No information leaked in the body beyond a generic `Unauthorized` message.
- [ ] 2.3 Export both middlewares from `src/utils/mod.rs`.

## 3. Backend — routing restructure

- [ ] 3.1 In `src/handlers/mod.rs`, move the existing `web_feed` registration into a guarded scope so `/channels/{channel_id}/feed.xml` is wrapped by `basic_auth`. Replace the current `configure(web_feed)` call with a scope that applies the middleware.
- [ ] 3.2 In `src/handlers/mod.rs`, restructure the `/api/1.0` scope so that `POST /login/`, `GET /logout/`, `GET /status/`, and `GET /session/` remain directly under `/api/1.0` (anonymous), and a new inner `web::scope("").wrap(require_session)` contains `channels::*`, `episodes::read_with_pagination`, and `config::get_config`. Remove the commented `//.wrap(Authentication)` line.

## 4. Backend — media files

- [ ] 4.1 Remove `af::Files::new("/media", "./audios")` from `main.rs`.
- [ ] 4.2 Register a guarded `web::scope("/media").wrap(basic_auth).service(af::Files::new("", "./audios"))` inside `handlers::config_services` so `/media/**` is served from the actix scope with the same middleware as the feed.

## 5. Backend — verification

- [ ] 5.1 `curl -i /api/1.0/channels/` (no cookie) → `401` with `CustomResponse` body containing `user: null`.
- [ ] 5.2 Log in via `POST /api/1.0/login/` with `admin` / `<admin_password>`, then `curl -i -b cookie /api/1.0/channels/` → `200` with channel list in `data`.
- [ ] 5.3 `curl -i /channels/1/feed.xml` (no credentials) → `401` with `WWW-Authenticate: Basic realm="u2vpodcast"`.
- [ ] 5.4 `curl -i -u admin:wrong /channels/1/feed.xml` → `401` identical to 5.3.
- [ ] 5.5 `curl -i -u admin:<admin_password> /channels/1/feed.xml` → `200` with `Content-Type: application/rss+xml; charset=utf-8` and a valid RSS body.
- [ ] 5.6 `curl -i /media/1/<yt_id>.mp3` (no credentials) → `401` with `WWW-Authenticate`. Repeat with valid Basic credentials → `200` and the MP3 body.
- [ ] 5.7 Open the SPA in a browser, log in, navigate to `/app/` and `/app/<channel_id>/`; confirm lists render and the `+page.ts` redirect path still triggers when the cookie is cleared.

## 6. Backend — client end-to-end

- [ ] 6.1 Subscribe the operator's podcast client to `https://<host>/channels/<id>/feed.xml` with `admin` / `<admin_password>` and confirm the feed loads and at least one episode downloads (validates that the client propagates Basic credentials to `<enclosure>` URLs).
- [ ] 6.2 If the client fails 6.1, pause and escalate the token-in-URL fallback; do not mark tasks 6.x complete until the client downloads successfully.

## 7. Release notes and rollback prep

- [ ] 7.1 Note in the changelog/release notes that any podcast client previously subscribed anonymously must be reconfigured with the admin username/password (one-time action).
- [ ] 7.2 Verify rollback path (revert `Cargo.toml`, `src/handlers/mod.rs`, `src/main.rs`, `src/utils/mod.rs`, and delete `src/utils/middleware.rs`) leaves the app buildable and behaviorally identical to before.