## ADDED Requirements

### Requirement: OIDC authorization code flow with PKCE

The system SHALL support login through an OIDC provider using the authorization
code flow with PKCE. `GET /api/1.0/login/oidc` SHALL redirect the browser to the
provider's authorization endpoint with `client_id`, the configured
`redirect_uri`, the `openid profile email` scopes, a random `state` value, and a
PKCE `code_challenge`. `GET /api/1.0/login/oidc/callback` SHALL validate the
returned `state` against the one issued, exchange the authorization code for
tokens using the PKCE verifier, and validate the id_token signature, issuer,
audience, and nonce before trusting it. On success the system SHALL establish
the same session cookie shape used by password login (`user_id`, `user_name`,
`user_role`, `user_active`) and redirect the browser to `/app/`.

#### Scenario: Successful OIDC login redirects to the provider

- **WHEN** a client requests `GET /api/1.0/login/oidc` and OIDC is enabled
- **THEN** the system responds with a `302` redirect to the provider's
  authorization endpoint carrying `client_id`, `redirect_uri`, `scope`, `state`,
  and `code_challenge`

#### Scenario: Valid callback creates a session and redirects to the app

- **WHEN** the provider redirects to the callback with a valid `code` and
  `state`, the token exchange succeeds, and the id_token validates
- **THEN** the system responds with a `302` redirect to `/app/` and a session
  cookie containing `user_id`, `user_name`, `user_role`, and `user_active`

#### Scenario: Invalid state on the callback is rejected

- **WHEN** the provider redirects to the callback with a `state` that does not
  match the one issued by the login endpoint
- **THEN** the system responds with `400 Bad Request` and does NOT create a
  session

### Requirement: OIDC users are provisioned just-in-time

When the id_token validates, the system SHALL resolve the identity to a `users`
table row. If a row exists whose `name` matches the OIDC identity (the `email`
claim), the system SHALL use it. If no row exists, the system SHALL create one
with `name` set to the `email` claim, `auth_method = 'oidc'`, an empty
`hashed_password`, `role = Admin`, and `active = true`, then create the session
for that row.

#### Scenario: Existing user logs in via OIDC

- **WHEN** an OIDC identity with email `user@example.com` matches an existing
  active user named `user@example.com`
- **THEN** the system creates a session for that existing row and does not
  create a duplicate

#### Scenario: Unknown identity is provisioned on first login

- **WHEN** an OIDC identity with email `user@example.com` has no matching
  `users` row
- **THEN** the system inserts a row with `name = user@example.com`,
  `auth_method = 'oidc'`, empty `hashed_password`, `role = Admin`, and
  `active = true`, and creates a session for it

### Requirement: OIDC mode disables password authentication

When `oidc.enabled` is `true`, the system SHALL reject every attempt to
authenticate with a password: `POST /api/1.0/login/` SHALL respond `401
Unauthorized` for any username/password pair, and the startup reseed of the
admin account from `admin_username` / `admin_password` SHALL be skipped.
Password-based HTTP Basic Auth on feeds and media SHALL also be rejected; only
API tokens are accepted for Basic Auth in this mode.

#### Scenario: Password login rejected while OIDC is enabled

- **WHEN** `oidc.enabled` is `true` and a client posts valid local credentials
  to `/api/1.0/login/`
- **THEN** the system responds `401 Unauthorized` and does not set a session
  cookie

#### Scenario: Admin reseed skipped while OIDC is enabled

- **WHEN** `oidc.enabled` is `true` and `admin_username` / `admin_password` are
  present in `config.yml`
- **THEN** the system starts without modifying the `users` table (no reseed)

### Requirement: OIDC-disabled mode blocks passwordless accounts

When `oidc.enabled` is `false`, the system SHALL NOT accept authentication from
any user whose `auth_method = 'oidc'`: `POST /api/1.0/login/` SHALL respond
`401` for such users, and HTTP Basic Auth SHALL reject them (no password and no
token verification). This prevents accounts without a password from being
entered when the OIDC provider is turned off.

#### Scenario: OIDC user cannot log in while OIDC is disabled

- **WHEN** `oidc.enabled` is `false` and a client posts credentials for a user
  whose `auth_method = 'oidc'`
- **THEN** the system responds `401 Unauthorized` and does not set a session
  cookie

#### Scenario: OIDC user cannot use Basic Auth while OIDC is disabled

- **WHEN** `oidc.enabled` is `false` and a client sends a Basic Auth header
  identifying an `auth_method = 'oidc'` user to `/channels/1/feed.xml`
- **THEN** the system responds `401 Unauthorized` with
  `WWW-Authenticate: Basic realm="u2vpodcast"`
