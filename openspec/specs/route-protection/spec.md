## Purpose

Server-side access control for the JSON API, the RSS feed, and the static media files, all backed by the single admin account seeded from `config.yml`, so that only authenticated callers can read channel metadata, episodes, feeds, and downloaded MP3s.

## Requirements

### Requirement: JSON API rejects requests without a valid session

The system SHALL reject every request to `/api/1.0/channels/*`, `/api/1.0/episodes/*`, and `/api/1.0/config/*` that does not carry a cookie session containing a non-empty `user_id` that resolves to an active user in the `users` table. When rejected, the system SHALL respond with HTTP status `401 Unauthorized` and a JSON body of the same shape as existing handlers (`CustomResponse` with `status: false`, `status_code: 401`, `user: null`, `data: null`), so the existing SvelteKit redirect flow keeps working unchanged.

#### Scenario: Anonymous request to a protected JSON endpoint
- **WHEN** a client sends `GET /api/1.0/channels/` with no cookie or with a cookie that has no `user_id`
- **THEN** the system responds `401 Unauthorized` and a body `{"status": false, "status_code": 401, "user": null, "data": null, "message": "Unauthorized"}`

#### Scenario: Authenticated request to a protected JSON endpoint
- **WHEN** a client sends `GET /api/1.0/channels/` with a cookie session carrying a `user_id` that maps to an active admin user
- **THEN** the system responds `200 OK` with the usual `CustomResponse` payload containing the channel list in `data`

### Requirement: Anonymous JSON endpoints remain reachable

The system SHALL keep `POST /api/1.0/login/`, `GET /api/1.0/logout/`, `GET /api/1.0/status/`, and `GET /api/1.0/session/` reachable without any session. A request to any of these without a cookie SHALL be handled by their existing handler logic and MUST NOT be rejected with `401` simply for lacking a session.

#### Scenario: Login without prior session
- **WHEN** a client sends `POST /api/1.0/login/` with a JSON body `{"username": "admin", "password": "<admin_password>}` and no cookie
- **THEN** the system responds `200 OK` and sets a session cookie containing `user_id`, `user_name`, `user_role`, and `user_active`

#### Scenario: Status check without session
- **WHEN** a client sends `GET /api/1.0/status/` with no cookie
- **THEN** the system responds `200 OK` and a `CustomResponse` body with `data: "Up and running"`

### Requirement: RSS feed requires a valid session or HTTP Basic Auth

The system SHALL accept a request to `/channels/{slug}/feed.xml` and `/{slug}/feed.xml` when **either** (a) the request carries a cookie session containing a `user_id` that resolves, via `from_session`, to an active row in the `users` table, **or** (b) the request carries a valid `Authorization: Basic` header whose credentials resolve, via `User::get_by_name` followed by `verify_password`, to an active row in the `users` table. When neither is present or both fail, the system SHALL respond with HTTP status `401 Unauthorized` and a `WWW-Authenticate: Basic realm="u2vpodcast"` response header, so podcast clients surface a credential prompt.

#### Scenario: Feed request without any credentials
- **WHEN** a client sends `GET /channels/1/feed.xml` with no cookie and no `Authorization` header
- **THEN** the system responds `401 Unauthorized` with `WWW-Authenticate: Basic realm="u2vpodcast"` and no feed body

#### Scenario: Feed request with valid Basic Auth credentials
- **WHEN** a client sends `GET /channels/1/feed.xml` with `Authorization: Basic <base64("admin:<admin_password>")>`
- **THEN** the system responds `200 OK` with `Content-Type: application/rss+xml; charset=utf-8` and the channel's RSS document in the body

#### Scenario: Feed request with valid session cookie only
- **WHEN** a browser that has a valid session cookie for the app sends `GET /channels/1/feed.xml` with no `Authorization` header
- **THEN** the system responds `200 OK` with the channel's RSS document, without sending a `WWW-Authenticate` header

#### Scenario: Feed request with wrong Basic Auth and invalid session
- **WHEN** a client sends `GET /channels/1/feed.xml` with `Authorization: Basic <base64("admin:wrong")>` and an expired or absent session
- **THEN** the system responds `401 Unauthorized` with `WWW-Authenticate: Basic realm="u2vpodcast"` and no feed body

### Requirement: Static media requires a valid session or HTTP Basic Auth

The system SHALL accept a request to `/media/**` when **either** (a) the request carries a valid cookie session (resolved via `from_session` to an active user) **or** (b) the request carries a valid `Authorization: Basic` header resolved in the same way as for the RSS feed. The same credential pair used for the feed and session login MUST also be valid for media. When neither is present or both fail, the system SHALL respond with `401 Unauthorized` and a `WWW-Authenticate: Basic realm="u2vpodcast"` header.

#### Scenario: Media request without any credentials
- **WHEN** a client sends `GET /media/1/abc123.mp3` with no cookie and no `Authorization` header
- **THEN** the system responds `401 Unauthorized` with `WWW-Authenticate: Basic realm="u2vpodcast"` and no audio body

#### Scenario: Media request with valid Basic Auth credentials
- **WHEN** a client sends `GET /media/1/abc123.mp3` with the valid Basic Auth header
- **THEN** the system responds `200 OK` with the MP3 file in the body and the appropriate `Content-Type`

#### Scenario: Media request with valid session cookie only (SPA audio playback)
- **WHEN** a browser that has a valid session cookie sends `GET /media/<slug>/<yt_id>.mp3` with no `Authorization` header (as the `<audio>` element does)
- **THEN** the system responds `200 OK` with the audio file, without sending a `WWW-Authenticate` header, so the browser does not show a credentials prompt

### Requirement: Feeds and media share the JSON API's credential store

The credential pair accepted for the RSS feed and for `/media/**` SHALL be exactly the one accepted for logging into the JSON API. The source of that pair is a single `users` table: either the admin account reseeded from `config.yml` (`admin_username` / `admin_password`) at startup via `User::default`, or — when both config credentials are absent — the pre-existing active row in the `users` table that is left untouched at startup. The system MUST NOT introduce a second credential store for feeds and media, regardless of provisioning mode.

#### Scenario: Same password used for API login and feed access
- **WHEN** an operator logs into `/api/1.0/login/` with `admin` / `<admin_password>` and, in the same session, also subscribes a podcast client to `/channels/1/feed.xml` using the same `admin` / `<admin_password>`
- **THEN** both requests are accepted and resolved to the same `users` row

#### Scenario: Feed and media credentials match stored user in config-less mode

- **WHEN** the system runs with both config credentials absent and the `users` table holds an active row with username `admin` and password `secret`
- **THEN** `GET /channels/1/feed.xml` and `GET /media/1/abc123.mp3` with `Authorization: Basic <base64("admin:secret")>` both return `200`, while `admin` with any other password returns `401`

### Requirement: 401 responses from the JSON API session guard preserve the existing response shape

When the JSON API session guard rejects a request with `401`, the response body SHALL be a `CustomResponse` JSON document with `status: false`, `status_code: 401`, `user: null`, `data: null`, and a non-empty `message`. The body SHALL be parseable by the existing SvelteKit `+page.ts` loaders (`JSON.parse` + `if (response.user == null)`) so that protected pages redirect to `/app/login?next=...` instead of breaking.

#### Scenario: Frontend redirect on session loss
- **WHEN** a browser loads `/app/channels/` with either a missing or an invalid session cookie and the page makes its `load()` fetch to `/api/1.0/channels/`
- **THEN** the API responds `401` with a `CustomResponse` body whose `user` field is `null`, and the SvelteKit `+page.ts` loader follows its existing code path returning `redirect(302, "/app/login?next=/app/")`

### Requirement: Admin account is the only accepted credential pair

Because the startup routine in `main.rs` either deletes all users and recreates only the admin from `config.yml` on every boot (seeded mode) or leaves the existing `users` table untouched when both config credentials are absent (stored mode), the route protection logic SHALL treat failed credential resolution (unknown username, inactive user, or wrong password) the same way and respond `401`, without distinguishing the failure mode in the response body or in log content beyond the existing `tracing::error!` call level. This requirement prevents leaking which usernames exist and applies identically in both provisioning modes.

#### Scenario: Unknown username is rejected identically to wrong password
- **WHEN** a client sends a Basic Auth header with username `notadmin` and any password to `/channels/1/feed.xml`
- **THEN** the system responds `401 Unauthorized` with `WWW-Authenticate: Basic realm="u2vpodcast"`, identical (status code and header) to the response it would give for `admin` with a wrong password

### Requirement: `with_authentication` config flag toggles the feed and media Basic Auth guard

The system SHALL read a boolean `with_authentication` field from `config.yml` at startup and use it to toggle the HTTP Basic Auth guard on the RSS feed (`/channels/{channel_id}/feed.xml`) and on `/media/**` only. When `with_authentication` is `true`, both surfaces SHALL require valid Basic Auth as described above. When `with_authentication` is `false`, both surfaces SHALL be served without any credential check (the pre-change public behavior). The JSON API session guard (`require_session` on `/api/1.0/channels|episodes|config/*`) is NOT affected by this flag and SHALL remain enforced regardless of the flag's value. The flag MUST be evaluated on every request (no startup caching that would prevent an operator from flipping the flag between restarts).

#### Scenario: Feed and media are public when the flag is false
- **WHEN** `config.yml` sets `with_authentication: false` and a client requests `/channels/1/feed.xml` or `/media/1/<yt_id>.mp3` with no `Authorization` header
- **THEN** the system responds `200` with the feed body (or the MP3 body) and no `WWW-Authenticate` header

#### Scenario: Feed and media are guarded when the flag is true
- **WHEN** `config.yml` sets `with_authentication: true` and a client requests `/channels/1/feed.xml` with no `Authorization` header
- **THEN** the system responds `401 Unauthorized` with `WWW-Authenticate: Basic realm="u2vpodcast"`

#### Scenario: API session guard is independent of the flag
- **WHEN** `config.yml` sets `with_authentication: false` and a client requests `/api/1.0/channels/` with no cookie
- **THEN** the system still responds `401` with the `CustomResponse` body (`user: null`), because `require_session` is not controlled by `with_authentication`

### Requirement: Client-side redirect on missing session

The frontend SHALL handle anonymous access to protected routes in the Vue SPA instead of server-side SvelteKit loaders. When a request to a protected JSON endpoint (`/api/1.0/channels/*`, `/api/1.0/channels/{id}/episodes/`, `/api/1.0/config/*`) returns `401` with `user: null`, or when the user has no session at all, the SPA SHALL redirect to `/login` preserving the intended destination as a `next` parameter for post-login return. The backend `401` contract (`CustomResponse` with `status: false`, `status_code: 401`, `user: null`, `data: null`) is unchanged.

#### Scenario: Frontend redirect on session loss
- **WHEN** a user without a valid session requests `/` and the SPA fetches `/api/1.0/channels/`, receiving `401` with `user: null`
- **THEN** the SPA redirects to `/login?next=/` instead of rendering the channel list

#### Scenario: Post-login return to the original destination
- **WHEN** an anonymous user was redirected to `/login?next=/42` and then logs in successfully
- **THEN** the SPA navigates to `/42` after a successful login

#### Scenario: Session expires during use of a protected route
- **WHEN** a user is browsing `/` and a subsequent API call (e.g., delete channel) returns `401` with `user: null`
- **THEN** the SPA clears its auth state and redirects to `/login`

### Requirement: Session claims are revalidated against the users table

Every request protected by session authorization (`RequireSession`, and the session branch of feed/media access) SHALL resolve the session `user_id` against the current `users` table on each request. If the row no longer exists or its `active` flag is `false`, the request SHALL be rejected with `401 Unauthorized`. The request SHALL NOT be authorized purely on stale cookie claims.

#### Scenario: Deleted user loses access immediately
- **WHEN** a user is deleted from the `users` table while holding a valid session cookie
- **THEN** the next protected request with that cookie returns `401 Unauthorized`

#### Scenario: Deactivated user loses access immediately
- **WHEN** a user's `active` flag is set to `false` while their session cookie is still valid
- **THEN** the next protected request with that cookie returns `401 Unauthorized`

#### Scenario: Active user keeps working
- **WHEN** a request carries a session cookie whose `user_id` resolves to an existing active row
- **THEN** the request is authorized and behaves exactly as before (no latency or shape change beyond the lookup)

#### Scenario: Reseeded admin invalidates old cookies
- **WHEN** the app starts in seeded mode (all user rows replaced by a fresh admin) and an old cookie from a previous run is used
- **THEN** the request is rejected with `401` instead of succeeding against a stale session

### Requirement: Session claims reflect the database row

When a session is validated, the effective `role`/`active`/`name` used by subsequent logic SHALL come from the database row (refreshed), so a role or activation change never lingers in the cookie beyond the first request after the change.

#### Scenario: Role claim refreshed from DB
- **WHEN** the database row's role differs from the value baked into the cookie at login
- **THEN** protected requests use the database role for any authorization decision
