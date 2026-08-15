## 1. Data model and config

- [ ] 1.1 Add `auth_method` column to `users` via migration (`ALTER TABLE users ADD COLUMN auth_method TEXT NOT NULL DEFAULT 'password'`) with down migration
- [ ] 1.2 Add `api_tokens` table via migration (`id`, `user_id`, `token_hash` unique, `prefix`, `created_at`, `last_used_at`, `revoked_at` nullable) with down migration
- [ ] 1.3 Add `AuthMethod` enum (`Password`, `Oidc`, sqlx lowercase) and `auth_method` field to `User` model (from_row, create, update)
- [ ] 1.4 Add `oidc` config block to `Config` (`enabled`, `issuer`, `client_id`, `client_secret`, `redirect_uri`) with `U2V_OIDC_*` env overrides after YAML load
- [ ] 1.5 Add `openidconnect` dependency to `Cargo.toml`
- [ ] 1.6 In `main.rs`, skip the admin reseed when `oidc.enabled` is true

## 2. API tokens

- [ ] 2.1 Add `ApiToken` model: generate (`u2v_` + 32 random bytes), sha256 hash, verify, list/revoke scoped by user
- [ ] 2.2 Add token endpoints (`POST /api/1.0/tokens/`, `GET /api/1.0/tokens/`, `DELETE /api/1.0/tokens/{id}/`) behind `RequireSession`, scoped to session `user_id`; regenerate = new token + revoke old transactionally
- [ ] 2.3 Register token routes in `src/handlers/mod.rs`

## 3. OIDC login flow

- [ ] 3.1 Add `oidc.rs` handler: `GET /api/1.0/login/oidc` builds authorize URL with PKCE S256, stores `state` (and verifier) in session, redirects
- [ ] 3.2 Add `oidc.rs` callback: validate state, exchange code, validate id_token (iss/aud/nonce), JIT-provision user, set session (4 keys), redirect to `/app/`
- [ ] 3.3 Register OIDC routes (public, outside `RequireSession`) in `src/handlers/mod.rs`

## 4. Auth mode switches

- [ ] 4.1 In `login.rs`, reject all `POST /api/1.0/login/` when `oidc.enabled` is true, and reject users with `auth_method = 'oidc'`
- [ ] 4.2 In `SessionOrBasicAuth`, branch on `oidc.enabled`: OIDC ON → verify API token; OIDC OFF → verify password and reject `auth_method = 'oidc'` users
- [ ] 4.3 Verify `with_authentication` flag behavior is unchanged (feeds/media guard toggle)

## 5. Frontend

- [ ] 5.1 Extend `GET /api/1.0/status/` to return `oidc_enabled` (public endpoint; config is behind RequireSession)
- [ ] 5.2 Add OIDC login button to `LoginView.vue` linking to `/api/1.0/login/oidc`, shown only when status reports `oidc_enabled: true`
- [ ] 5.3 Add `TokenDialog.vue`: header button beside Logout opens `AppDialog`; list own token metadata (prefix, created, last use) with Generate/Regenerate (via ConfirmDialog) and Revoke (via ConfirmDialog)- [ ] 5.4 One-time token display: full token held in memory ref only (never localStorage), Copy button, cleared on dialog close; regenerate revokes previous token server-side
- [ ] 5.5 Add token endpoints to `frontend/src/lib/api/client.ts` (list, generate, revoke)

## 6. Verify

- [ ] 6.1 `cargo build` and `cargo test` pass
- [ ] 6.2 `pnpm test`, `pnpm run lint`, and `pnpm run build` pass in `frontend/`
- [ ] 6.3 Manual (any OIDC provider; Authentik as reference): OIDC ON → password login blocked, OIDC login works, Basic Auth with token works, token regenerate revokes old
- [ ] 6.4 Manual: OIDC OFF → password login works, OIDC-marked user cannot log in, token not accepted
