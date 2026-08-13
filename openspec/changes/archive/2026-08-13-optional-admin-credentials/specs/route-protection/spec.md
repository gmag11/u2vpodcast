## MODIFIED Requirements

### Requirement: Feeds and media share the JSON API's credential store

The credential pair accepted for the RSS feed and for `/media/**` SHALL be exactly the one accepted for logging into the JSON API. The source of that pair is a single `users` table: either the admin account reseeded from `config.yml` (`admin_username` / `admin_password`) at startup via `User::default`, or — when both config credentials are absent — the pre-existing active row in the `users` table that is left untouched at startup. The system MUST NOT introduce a second credential store for feeds and media, regardless of provisioning mode.

#### Scenario: Same password used for API login and feed access

- **WHEN** an operator logs into `/api/1.0/login/` with `admin` / `<password>` and, in the same session, also subscribes a podcast client to `/channels/1/feed.xml` using the same `admin` / `<password>`
- **THEN** both requests are accepted and resolved to the same `users` row

#### Scenario: Feed and media credentials match stored user in config-less mode

- **WHEN** the system runs with both config credentials absent and the `users` table holds an active row with username `admin` and password `secret`
- **THEN** `GET /channels/1/feed.xml` and `GET /media/1/abc123.mp3` with `Authorization: Basic <base64("admin:secret")>` both return `200`, while `admin` with any other password returns `401`

### Requirement: Admin account is the only accepted credential pair

Because the startup routine in `main.rs` either deletes all users and recreates only the admin from `config.yml` on every boot (seeded mode) or leaves the existing `users` table untouched when both config credentials are absent (stored mode), the route protection logic SHALL treat failed credential resolution (unknown username, inactive user, or wrong password) the same way and respond `401`, without distinguishing the failure mode in the response body or in log content beyond the existing `tracing::error!` call level. This requirement prevents leaking which usernames exist and applies identically in both provisioning modes.

#### Scenario: Unknown username is rejected identically to wrong password

- **WHEN** a client sends a Basic Auth header with username `notadmin` and any password to `/channels/1/feed.xml`
- **THEN** the system responds `401 Unauthorized` with `WWW-Authenticate: Basic realm="u2vpodcast"`, identical (status code and header) to the response it would give for `admin` with a wrong password
