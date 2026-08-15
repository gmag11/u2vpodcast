## MODIFIED Requirements

### Requirement: RSS feed requires a valid session or HTTP Basic Auth

The system SHALL accept a request to `/channels/{slug}/feed.xml` and `/{slug}/feed.xml` when **either** (a) the request carries a cookie session containing a `user_id` that resolves, via `from_session`, to an active row in the `users` table, **or** (b) the request carries a valid `Authorization: Basic` header resolved against the `users` table: when OIDC is disabled, credentials resolve via `User::get_by_name` followed by `verify_password` for users whose `auth_method = 'password'` (users with `auth_method = 'oidc'` SHALL be rejected); when OIDC is enabled, the password portion SHALL be verified as an API token for the identified user and password verification SHALL NOT be performed. When neither is present or both fail, the system SHALL respond with HTTP status `401 Unauthorized` and a `WWW-Authenticate: Basic realm="u2vpodcast"` response header, so podcast clients surface a credential prompt.

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

#### Scenario: Feed access with a valid token while OIDC is enabled
- **WHEN** `oidc.enabled` is `true` and a client sends a Basic Auth header whose password portion is a valid API token for the identified user
- **THEN** the system responds `200 OK` with the channel's RSS document

#### Scenario: OIDC user rejected via password path while OIDC is disabled
- **WHEN** `oidc.enabled` is `false` and a client sends a Basic Auth header identifying a user whose `auth_method = 'oidc'` (with any password)
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
