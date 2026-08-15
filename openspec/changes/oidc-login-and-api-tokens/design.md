## Context

The app authenticates against a single local `users` table with argon2
passwords. `POST /api/1.0/login/` sets a cookie session with four keys
(`user_id`, `user_name`, `user_role`, `user_active`); `from_session` reads them
back. Two middleware guards protect routes: `RequireSession` for the JSON API
and `SessionOrBasicAuth` for the RSS feeds and `/media/**`. The startup
routine reseeds the admin from `admin_username`/`admin_password` when both are
present. The `role` field is decorative — no endpoint consults it for
authorization. There is no per-user content separation: every authenticated
user sees everything.

Goal: let operators authenticate through an OIDC provider instead of local
passwords, and give OIDC users (who have no password) per-user API tokens so
podcast apps and scripts can still use HTTP Basic Auth on feeds and media.

## Goals / Non-Goals

**Goals:**
- OIDC authorization code flow with PKCE, just-in-time user provisioning, and
  the existing session shape.
- Explicit `auth_method` column (`password` | `oidc`) on `users`.
- Mode switch: OIDC enabled → password auth fully disabled (login form, HTTP
  Basic); OIDC disabled → password auth as today and OIDC-marked users blocked.
- Per-user API tokens (one active per user) for HTTP Basic Auth, SHA-256
  hashed, `u2v_` prefixed, never expiring, with list/generate/regenerate/revoke.
- OIDC config in `config.yml` with `U2V_OIDC_*` env overrides.

**Non-Goals:**
- No per-user content separation; all users share full access.
- No token expiry, scopes, or OAuth2-only tokens (no Bearer scheme).
- No refresh-token persistence or server-side token rotation.
- No OIDC logout / `end_session_endpoint` integration.
- No changes to the `RequireSession` JSON API guard semantics.

## Decisions

### 1. Explicit `auth_method` column over password-emptiness heuristic

Add `auth_method TEXT NOT NULL DEFAULT 'password'` to `users` (sqlx
`AuthMethod` enum mirroring `Role`). OIDC users get `hashed_password = ""` and
`auth_method = 'oidc'`. Login gates check `auth_method` first, never the empty
password. Rationale: explicit and safe — an empty-password heuristic could be
tricked by a malformed row; a column makes the policy declarative. Migration is
additive (`DEFAULT 'password'`), existing rows are untouched.

### 2. Mode-based branching, not per-user branching, in middleware

`SessionOrBasicAuth` decides by `oidc.enabled`, not by `auth_method`:

```
Basic Auth branch:
  user = get_by_name(username)
  if oidc.enabled:
      verify_api_token(password_part)          # token is the password
  else:
      if user.auth_method == 'oidc': reject    # no passwordless entry
      verify_password(password_part)
```

Rationale: with OIDC ON there is exactly one HTTP auth credential type (token)
and with OIDC OFF exactly one (password). Branching per-user would create a
mixed mode the operator never asked for. The `auth_method` check still exists in
the OFF branch to satisfy the "no login without password" rule.

### 3. Token model: one active token per user, sha256, `u2v_` prefix

`api_tokens` table: `id`, `user_id` (FK), `token_hash` (unique), `prefix`
(`u2v_` + first 8 chars for display), `created_at`, `last_used_at`, plus a
`revoked_at` nullable column or row deletion. Decision: single active token —
"regenerate" = insert new row + mark old revoked in one transaction. Token value
generated as 32 random bytes (`u2v_` + base64url). Full value returned once in
the creation response. Rationale: matches the user's "one token per user with
regenerate" requirement; avoids scope/expiry machinery that adds surface without
need.

### 4. OIDC user identity = email claim as `users.name`

JIT provisioning: lookup `users` by `name = email_claim`; create with
`role = Admin`, `active = true`, `auth_method = 'oidc'`, empty
`hashed_password` if missing. Rationale: `name` is already the unique login
identifier used by `get_by_name` and by the Basic Auth username. Trade-off:
if the email changes at the IdP, a new user row is provisioned. Accepted —
shared-content model makes orphan rows harmless.

### 5. `openidconnect` crate with PKCE

Use `openidconnect` (rust) — de-facto standard: discovery, PKCE, id_token
validation (issuer/audience/nonce), token exchange. `state` stored in the
session cookie (`oidc_state`) and validated on callback; PKCE `code_challenge`
S256. Rationale: hand-rolling OIDC is a security anti-pattern; the crate
encodes the RFC 8252 checks.

### 6. Config + env overrides

```yaml
oidc:
  enabled: false
  issuer: ...
  client_id: ...
  client_secret: ...
  redirect_uri: https://host/api/1.0/login/oidc/callback
```

`Config` gains an `oidc` block; after YAML load, `U2V_OIDC_*` env vars override
fields (env > yaml), via `std::env` — no new config crate. `client_secret` is
the field most likely to come from env only. Startup: if `oidc.enabled`, skip
the admin reseed.

### 7. Token endpoints scoped to session user

`POST /api/1.0/tokens/`, `GET /api/1.0/tokens/`,
`DELETE /api/1.0/tokens/{id}/` all require `RequireSession` and filter by the
session `user_id`. Cross-user access returns `404`. Regenerate reuses `POST` —
a user with an existing token gets a new one and the old is revoked.

## Risks / Trade-offs

- [OIDC IdP down → nobody can log in] → Accepted (operator choice); documented
  in the proposal. OIDC off restores password auth.
- [Email claim changes → orphan OIDC user row] → Accepted; shared-content model
  means orphans have no data; operator can delete via existing user tooling.
- [Token leak grants full access (no scopes)] → Mitigation: token hashed at
  rest, shown once, revocable instantly, `last_used_at` for detection.
- [Migrating an existing password user to OIDC] → Out of scope: switching
  requires OIDC on (password disabled) so password users are frozen until they
  have an IdP identity; operators should pre-provision `name = email` rows if
  they want continuity.
- [Basic Auth username for OIDC users is the email] → Podcast apps store
  `email:token`; document it. If email contains characters incompatible with a
  client's Basic handling, the user must rely on the web session instead.

## Migration Plan

1. Deploy migration: `ALTER TABLE users ADD COLUMN auth_method` +
   `CREATE TABLE api_tokens` (additive, backward compatible).
2. Release with `oidc.enabled: false` (default) — behavior identical to today.
3. Operator enables OIDC via config/env; password auth disables automatically.
4. Rollback: set `oidc.enabled: false` — password auth returns; OIDC-marked
   users are blocked until the provider is back or rows are migrated.
5. Tokens persist across rollback; they are simply not accepted while OIDC is
   off.

## Open Questions

- Whether `last_used_at` should be shown in the token list (already stored).

## OIDC Provider

Provider-agnostic: the design targets standard OIDC, not a specific vendor. Each
deployment configures its own single issuer in `oidc.issuer`;
`openidconnect`'s `CoreClient::from_provider_metadata` discovers endpoints from
`{issuer}/.well-known/openid-configuration`, so Authentik, Keycloak, Google,
Entra ID, etc. work with no code changes. No provider-specific URLs, claims, or
behaviors are assumed. The identity is the `email` claim, which Authentik and
all mainstream providers deliver with the `openid profile email` scopes.

The reference deployment used for manual acceptance tests (tasks 6.3/6.4) is
Authentik, but that is a test fixture choice, not a design constraint: the
same tests run against any OIDC-compliant provider. Multiple simultaneous
providers (per-user IdP selection) are out of scope — one issuer per deployment.

## Frontend Token Management

Decided: header dialog (Option A). `AppHeader` gains an "API Token" button
beside Logout that opens an `AppDialog` ("API tokens") reusing the existing
radix-vue dialog and `ConfirmDialog` patterns — no router changes, no nav
links. The dialog lists the current user's token metadata (prefix, created,
last use) with Generate/Regenerate/Revoke actions; regenerating and revoking
go through `ConfirmDialog`.

The full token is returned once, held in an in-memory ref only (never
localStorage), shown in a highlighted panel with a Copy button, and cleared on
dialog close or "Done" — if the user did not copy it, they regenerate (the
previous token is already revoked server-side on POST).

The public OIDC flag for `LoginView` comes from the public status endpoint:
`GET /api/1.0/status/` gains an `oidc_enabled` field (the config endpoint is
behind `RequireSession`, so it cannot serve anonymous login pages).
`LoginView` fetches it in `onMounted` and conditionally renders the OIDC login
button. A future `/settings` view can absorb the dialog if more settings
appear.
