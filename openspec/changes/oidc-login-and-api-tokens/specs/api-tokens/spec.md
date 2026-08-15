## ADDED Requirements

### Requirement: Users can generate a personal API token

The system SHALL let an authenticated user create a token for their own account
via `POST /api/1.0/tokens/`. The token SHALL be a random value of at least 32
bytes, prefixed with `u2v_`, returned in full only in the creation response,
and stored only as its SHA-256 hash. A user SHALL have at most one active
token at a time; creating a new one SHALL revoke any previous token of the same
user.

#### Scenario: Token is returned once and stored hashed

- **WHEN** an authenticated user posts to `/api/1.0/tokens/`
- **THEN** the response contains the full `u2v_`-prefixed token and the database
  stores only its SHA-256 hash

#### Scenario: Regenerating a token revokes the previous one

- **WHEN** a user with an existing token creates a new token
- **THEN** the old token stops working immediately and only the new token is
  accepted

### Requirement: Users can list and revoke their own tokens

The system SHALL let an authenticated user list their own tokens via
`GET /api/1.0/tokens/` (metadata only: id, prefix, creation time, last use; the
full token is never returned again) and revoke a token via
`DELETE /api/1.0/tokens/{id}/`. A user SHALL only be able to list and revoke
their own tokens; attempts to access another user's token SHALL respond `404
Not Found`.

#### Scenario: User lists their own tokens

- **WHEN** an authenticated user requests `GET /api/1.0/tokens/`
- **THEN** the response lists only that user's tokens with metadata and never
  the full token values

#### Scenario: Revoked token stops working

- **WHEN** a user revokes a token via `DELETE /api/1.0/tokens/{id}/`
- **THEN** HTTP Basic Auth using that token responds `401 Unauthorized`

#### Scenario: User cannot see another user's token

- **WHEN** a user requests the token list or deletes a token belonging to a
  different user
- **THEN** the system responds `404 Not Found`

### Requirement: API tokens authenticate HTTP Basic Auth

The system SHALL accept a token as the password portion of an HTTP Basic Auth
header (`Authorization: Basic base64("username:TOKEN")`) on the RSS feed and
`/media/**` surfaces when OIDC is enabled (`oidc.enabled: true`). The username
SHALL identify the user row (`name`); the system SHALL verify the token by
hashing the presented value and comparing against the stored hash of an active,
non-revoked token belonging to that user. When OIDC is disabled, token
verification SHALL NOT be performed.

#### Scenario: Feed access with a valid token

- **WHEN** `oidc.enabled` is `true` and a client sends a Basic Auth header with
  a valid username and a valid token to `/channels/1/feed.xml`
- **THEN** the system responds `200 OK` with the feed body

#### Scenario: Feed access with an invalid token

- **WHEN** `oidc.enabled` is `true` and a client sends a Basic Auth header with
  a wrong token
- **THEN** the system responds `401 Unauthorized` with
  `WWW-Authenticate: Basic realm="u2vpodcast"`

#### Scenario: Token is not accepted while OIDC is disabled

- **WHEN** `oidc.enabled` is `false` and a client sends a Basic Auth header with
  a token that would otherwise be valid
- **THEN** the system responds `401 Unauthorized` (token verification is not
  performed)
