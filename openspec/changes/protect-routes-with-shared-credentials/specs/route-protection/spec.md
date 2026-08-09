## Purpose

Server-side access control for the JSON API, the RSS feed, and the static media files, all backed by the single admin account seeded from `config.yml`, so that only authenticated callers can read channel metadata, episodes, feeds, and downloaded MP3s.

## ADDED Requirements

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

### Requirement: RSS feed requires HTTP Basic Auth

The system SHALL reject every request to `/channels/{channel_id}/feed.xml` that does not carry a valid `Authorization: Basic` header whose credentials resolve, via `User::get_by_name` followed by `verify_password` (argon2), to an active row in the `users` table. When rejected, the system SHALL respond with HTTP status `401 Unauthorized` and a `WWW-Authenticate: Basic realm="u2vpodcast"` response header, so podcast clients surface a credential prompt.

#### Scenario: Feed request without credentials
- **WHEN** a client sends `GET /channels/1/feed.xml` with no `Authorization` header
- **THEN** the system responds `401 Unauthorized` with `WWW-Authenticate: Basic realm="u2vpodcast"` and no feed body

#### Scenario: Feed request with wrong password
- **WHEN** a client sends `GET /channels/1/feed.xml` with `Authorization: Basic <base64("admin:wrong")>`
- **THEN** the system responds `401 Unauthorized` with `WWW-Authenticate: Basic realm="u2vpodcast"` and no feed body

#### Scenario: Feed request with correct credentials
- **WHEN** a client sends `GET /channels/1/feed.xml` with `Authorization: Basic <base64("admin:<admin_password>")>`
- **THEN** the system responds `200 OK` with `Content-Type: application/rss+xml; charset=utf-8` and the channel's RSS document in the body

### Requirement: Static media requires HTTP Basic Auth

The system SHALL reject every request to `/media/**` that does not carry a valid `Authorization: Basic` header resolved in the same way as for the RSS feed. The same credential pair used for the feed MUST also be valid for media. When rejected, the system SHALL respond with `401 Unauthorized` and a `WWW-Authenticate: Basic realm="u2vpodcast"` header.

#### Scenario: Media request without credentials
- **WHEN** a client sends `GET /media/1/abc123.mp3` with no `Authorization` header
- **THEN** the system responds `401 Unauthorized` with `WWW-Authenticate: Basic realm="u2vpodcast"` and no audio body

#### Scenario: Media request with correct credentials
- **WHEN** a client sends `GET /media/1/abc123.mp3` with the valid Basic Auth header
- **THEN** the system responds `200 OK` with the MP3 file in the body and the appropriate `Content-Type`

### Requirement: Feeds and media share the JSON API's credential store

The credential pair accepted for the RSS feed and for `/media/**` SHALL be exactly the one accepted for logging into the JSON API, i.e. the admin account seeded from `config.yml` (`admin_username` / `admin_password`) at startup via `User::default`. The system MUST NOT introduce a second credential store for feeds and media.

#### Scenario: Same password used for API login and feed access
- **WHEN** an operator logs into `/api/1.0/login/` with `admin` / `<admin_password>` and, in the same session, also subscribes a podcast client to `/channels/1/feed.xml` using the same `admin` / `<admin_password>`
- **THEN** both requests are accepted and resolved to the same `users` row

### Requirement: 401 responses from the JSON API session guard preserve the existing response shape

When the JSON API session guard rejects a request with `401`, the response body SHALL be a `CustomResponse` JSON document with `status: false`, `status_code: 401`, `user: null`, `data: null`, and a non-empty `message`. The body SHALL be parseable by the existing SvelteKit `+page.ts` loaders (`JSON.parse` + `if (response.user == null)`) so that protected pages redirect to `/app/login?next=...` instead of breaking.

#### Scenario: Frontend redirect on session loss
- **WHEN** a browser loads `/app/channels/` with either a missing or an invalid session cookie and the page makes its `load()` fetch to `/api/1.0/channels/`
- **THEN** the API responds `401` with a `CustomResponse` body whose `user` field is `null`, and the SvelteKit `+page.ts` loader follows its existing code path returning `redirect(302, "/app/login?next=/app/")`

### Requirement: Admin account is the only accepted credential pair

Because the startup routine in `main.rs` deletes all users and recreates only the admin from `config.yml` on every boot, the route protection logic SHALL treat failed credential resolution (unknown username, inactive user, or wrong password) the same way and respond `401`, without distinguishing the failure mode in the response body or in log content beyond the existing `tracing::error!` call level. This requirement prevents leaking which usernames exist.

#### Scenario: Unknown username is rejected identically to wrong password
- **WHEN** a client sends a Basic Auth header with username `notadmin` and any password to `/channels/1/feed.xml`
- **THEN** the system responds `401 Unauthorized` with `WWW-Authenticate: Basic realm="u2vpodcast"`, identical (status code and header) to the response it would give for `admin` with a wrong password