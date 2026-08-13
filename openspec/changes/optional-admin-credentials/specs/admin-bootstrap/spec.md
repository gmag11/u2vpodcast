## ADDED Requirements

### Requirement: Admin bootstrap honors stored database credentials when config omits both admin credentials

The system SHALL treat `admin_username` and `admin_password` in `config.yml` as optional for the purpose of database bootstrap. When **both** fields are absent, empty, or null in `config.yml`, the system SHALL NOT modify the `users` table at startup: it SHALL NOT delete existing rows and SHALL NOT create or overwrite the admin account. In that mode, authentication for the JSON API session login, the RSS feed Basic Auth, and `/media/**` SHALL resolve credentials against the existing `users` table rows via `User::get_by_name` plus password verification, exactly as in the seeded mode. If the database holds no active user in that mode, every authenticated surface SHALL reject requests with `401 Unauthorized`; the system MUST NOT invent credentials from config.

#### Scenario: Both config credentials absent and a user exists in the database

- **WHEN** `config.yml` contains no `admin_username` and no `admin_password`, and the `users` table already contains an active row (e.g. `admin` with a hashed password)
- **THEN** the system starts successfully without deleting or altering the `users` table, and `POST /api/1.0/login/` with the stored username and password returns `200 OK` and sets a session cookie

#### Scenario: Both config credentials absent and the database is empty

- **WHEN** `config.yml` contains no `admin_username` and no `admin_password`, and the `users` table has no rows
- **THEN** the system starts successfully, leaves the table empty, and a login attempt with any credentials returns `401 Unauthorized`; the feed and media surfaces also return `401` (unless `with_authentication: false`)

#### Scenario: Pre-existing user survives a restart in config-less mode

- **WHEN** the system runs in config-less mode, an operator changes the admin password directly in the database, and then the service restarts without adding credentials to `config.yml`
- **THEN** the changed password remains valid after restart and the original one no longer works

### Requirement: Admin bootstrap seeds from config only when both credentials are present

The system SHALL preserve the existing bootstrap behavior only when **both** `admin_username` and `admin_password` are present and non-empty in `config.yml`: on every startup it SHALL delete all rows from the `users` table and recreate the single admin account from the config values via `User::default`. If only one of the two fields is present (or empty), the system SHALL ignore both config credentials entirely and run in stored mode: it SHALL NOT modify the `users` table. This mode SHALL be the default and MUST remain backward compatible with existing configurations that set both fields.

#### Scenario: Both credentials present in config

- **WHEN** `config.yml` sets both `admin_username: admin` and `admin_password: nimda`
- **THEN** on startup all previous `users` rows are removed and a single active admin row with username `admin` and the argon2 hash of `nimda` is created, and logging in with those credentials succeeds

#### Scenario: Only one credential present in config

- **WHEN** `config.yml` sets `admin_username` but omits `admin_password` (or vice versa)
- **THEN** both config credentials are ignored, the `users` table is left untouched, and authentication resolves against the existing database rows as in stored mode

### Requirement: Startup logs the active provisioning mode

The system SHALL emit a log line at startup that states whether it seeded the admin from `config.yml` or kept the existing database users, using the existing `tracing::info!` facility, so operators can confirm which provisioning mode is active.

#### Scenario: Config-less startup logs kept-database mode

- **WHEN** the system starts with both admin credentials absent from `config.yml`
- **THEN** the log contains an informational entry indicating that existing database users were preserved

#### Scenario: Seeded startup logs seed mode

- **WHEN** the system starts with admin credentials present in `config.yml`
- **THEN** the log contains an informational entry indicating that the admin was reseeded from configuration

### Requirement: The database stores a hash of the password, never plaintext

The system SHALL NOT persist the admin password in plaintext. When the admin is reseeded from `config.yml` in seeded mode, the `users` row SHALL store only the argon2 hash of the config password (via the existing `User::new` → `token_utils::hash_password` path), and the raw password SHALL never be written to the database, logs, or response bodies. In stored mode the system SHALL rely exclusively on already-hashed rows in the `users` table and SHALL NOT re-hash or rewrite any password. Verification SHALL always compare a supplied password against the stored hash with `verify_password`.

#### Scenario: Seeded admin row contains a hash

- **WHEN** the system starts in seeded mode with `admin_password: nimda`
- **THEN** the `hashed_password` column of the admin row contains an argon2 hash that does not equal `nimda`, and a login with `nimda` succeeds while a login with `nimda2` fails

#### Scenario: No plaintext password ever returned by the API

- **WHEN** any handler serializes a `User` row (e.g. the session payload)
- **THEN** the response contains the username and role but never the raw password or the hashed value
