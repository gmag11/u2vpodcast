## 1. Middleware

- [x] 1.1 In `src/utils/middleware.rs`, add a `SessionOrBasicAuth` struct implementing `Transform` and a `SessionOrBasicAuthMiddleware` service implementing `Service<ServiceRequest>`. The `call` method SHALL: (a) read `with_authentication` from `AppState` and pass if `false`; (b) extract the session and call `from_session` — pass on success; (c) extract `BasicAuth` from the request header, validate username/password against `User::get_by_name` + `check_password` — pass on success; (d) if both fail, return `401` with `WWW-Authenticate: Basic realm=\"u2vpodcast\"`.

## 2. Route wiring

- [x] 2.1 In `src/handlers/feed.rs`, replace the `BasicAuthGuard` import with `SessionOrBasicAuth` and replace `.wrap(BasicAuthGuard)` with `.wrap(SessionOrBasicAuth)` on both feed routes (`/channels/{slug}/feed.xml` and `/{slug}/feed.xml`).
- [x] 2.2 In `src/handlers/mod.rs`, replace the `BasicAuthGuard` import with `SessionOrBasicAuth` and replace `.wrap(BasicAuthGuard)` on the `/media` scope with `.wrap(SessionOrBasicAuth)`. Keep the `RequireSession` import and usage unchanged.

## 3. Verification

- [x] 3.1 `cargo build` in the container and confirm the image runs.
- [x] 3.2 Log into the SPA and open an episode page with an audio player. Confirm the audio loads without a Basic Auth prompt (the `<audio>` element receives `200` via the session cookie).
- [x] 3.3 From the same browser, access `/channels/<slug>/feed.xml` directly. Confirm the feed renders (`200`) without a Basic Auth prompt.
- [x] 3.4 Using a fresh incognito window or `curl` without cookies, access `/channels/<slug>/feed.xml`. Confirm the server responds `401` with `WWW-Authenticate: Basic realm="u2vpodcast"`, and after supplying valid Basic Auth credentials the feed returns `200`.
- [x] 3.5 Using `curl` without cookies, access `/media/<slug>/<yt_id>.mp3`. Confirm `401` without credentials and `200` with valid Basic Auth.
- [x] 3.6 Verify that the JSON API guard (`RequireSession`) is unaffected: accessing `/api/1.0/channels/` without a session still returns `401` with the `CustomResponse` JSON body.
