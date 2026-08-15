## Why

The app only supports a single local admin account authenticated with a
password. Operators want to authenticate through an OIDC provider (e.g.
Authelia, Keycloak, Google) so the identity source is centralized and there is
no local credential to manage. OIDC users have no password, so they need
per-user tokens to use HTTP Basic Auth on the RSS feeds and `/media/**`
endpoints from podcast apps and scripts. Content is shared: every authenticated
user sees everything, there is no per-user content separation.

## What Changes

- Add OIDC login flow: `GET /api/1.0/login/oidc` starts the authorization code
  flow with PKCE, and `GET /api/1.0/login/oidc/callback` exchanges the code,
  validates the id_token, and provisions the user (just-in-time) in the
  `users` table.
- Add an `auth_method` column to the `users` table with values `password` and
  `oidc` so the credential type is explicit instead of inferred.
- When OIDC is enabled (`oidc.enabled: true`), password authentication SHALL be
  disabled entirely: `POST /api/1.0/login/` rejects all attempts, and HTTP
  Basic Auth accepts only API tokens. The startup admin reseed from
  `admin_username`/`admin_password` is skipped.
- When OIDC is disabled, password authentication works as today; users with
  `auth_method = 'oidc'` cannot log in through any surface and token
  verification is disabled, so no account can ever be entered without a
  password in that mode.
- Add per-user API tokens: each user can list, generate, regenerate, and revoke
  their own tokens. Tokens never expire, are stored as SHA-256 hashes, use the
  `u2v_` prefix, and are shown in full only once at creation.
- Add OIDC configuration to `config.yml` with environment variable overrides
  (`U2V_OIDC_*`).
- OIDC users are created with `role = Admin` and share the same permissions as
  existing users; the `role` field remains decorative.

## Capabilities

### New Capabilities

- `oidc-login`: OIDC authorization code flow with PKCE, just-in-time user
  provisioning, and session creation; the mode switch that disables password
  auth when OIDC is enabled.
- `api-tokens`: per-user API tokens for HTTP Basic Auth, with list, generate,
  regenerate, and revoke operations; SHA-256 hashing and `u2v_` prefix.

### Modified Capabilities

- `route-protection`: HTTP Basic Auth behavior changes — when OIDC is enabled
  the token replaces the password for Basic Auth, and when OIDC is disabled
  `auth_method = 'oidc'` users are rejected everywhere.
- `admin-bootstrap`: the startup reseed from config credentials is skipped when
  OIDC is enabled.

## Impact

- `src/models/user.rs`: add `auth_method` field and column; hashed_password
  empty for OIDC users.
- `src/models/api_token.rs` (new): token model, SHA-256 hashing, verify.
- `src/handlers/login.rs`: block password login when OIDC enabled or user is
  `auth_method = 'oidc'`.
- `src/handlers/oidc.rs` (new): authorization redirect and callback handlers.
- `src/handlers/tokens.rs` (new): token CRUD endpoints.
- `src/utils/middleware.rs`: token verification in `SessionOrBasicAuth`;
  reject OIDC users on password path.
- `src/models/config.rs`: OIDC config block + env overrides.
- `src/main.rs`: register routes, skip admin reseed when OIDC enabled.
- `migrations/`: add `auth_method` column and `api_tokens` table.
- `Cargo.toml`: add `openidconnect` dependency.
- `frontend/src/views/LoginView.vue`: OIDC login button when enabled.
- Frontend token management UI for the current user.
- No per-user content separation; all authenticated users share full access.
