# Design

## Context

See `proposal.md` - Why. Relevant current state:

- `src/handlers/mod.rs` wraps `/media/**`, `/feed.xml`, `/channels/{slug}/feed.xml`, `/{slug}/feed.xml`, and `/images/**` with `SessionOrBasicAuth`, and the JSON API under `/api/1.0/*` with `RequireSession`. `SessionOrBasicAuth` is gated by `config.with_authentication` and is the only credential path these surfaces understand.
- `src/handlers/media.rs` already implements byte-range (including open-ended and suffix), `ETag`, `Last-Modified`, conditional `304`, and the SponsorBlock/original representation selection. That streaming/validation logic is embedded in `serve_media`, which takes a relative `/media` path and resolves it.
- `src/models/config.rs` loads and validates `config.yml`. `secret_key` is required, validated to at least 64 bytes, and stable across restarts (it signs session cookies).
- `src/handlers/feed.rs` shows the pattern for reading an episode by `yt_id` and selecting its current media (`Episode::read_by_yt_id_with_channel` + `selected_media`).
- The SPA is served as static files under `/app` by an `actix-files` default handler, so any unknown `/app/...` path returns `index.html`. `frontend/src/router/index.ts` currently treats `meta.public` as "guest-only": an authenticated user visiting a public route is redirected to `channels`, which is correct for `/login` but wrong for a share page.
- `frontend/src/lib/api/client.ts` targets the authenticated `/api/1.0/*` API and always sends `credentials: 'include'`.

## Goals / Non-Goals

**Goals:**

- Authorize a public share surface with a self-contained signed token and no new persistent state.
- Reuse the existing media streaming and representation-selection behavior so range/seek/`ETag` semantics are identical to protected playback.
- Keep the existing protected surfaces and `with_authentication` semantics untouched.
- Make the public page usable by an anonymous visitor and not disruptive to an already-authenticated one.

**Non-Goals:**

- Per-link revocation, audit trail, or analytics (only `secret_key` rotation revokes).
- An RSS feed or any podcast-client surface for a shared episode.
- Sharing a channel or playlist.
- Persisting or syncing the anonymous visitor's playback progress.
- Any change to how episodes, media, or feeds are protected for existing routes.

## Decisions

### Decision: Stateless HMAC token derived from `secret_key`

Token layout:

```
payload = "v1|{yt_id}|{exp_unix}"
K       = HMAC-SHA256(secret_key, "u2vpodcast/share/v1")   // domain separation
sig     = HMAC-SHA256(K, payload)                          // lowercase hex
token   = "{exp_unix}-{yt_id}-{sig}"
```

Verification recomputes the signature over `"v1|{yt_id}|{exp_unix}"` (with the `yt_id` taken from the token and, for resource routes, cross-checked) and compares in constant time; the token is rejected when the signature fails or `exp_unix` is in the past. The `yt_id` characters (`[A-Za-z0-9_-]`) and the numeric expiry make the token URL-safe without a base64 dependency.

Alternatives considered:

- **A `share_links` table with opaque tokens.** Rejected: it adds a migration, a cleanup concern, and state the feature does not need for its chosen revocation model.
- **A separate `share_secret` config value.** Rejected: another secret to provision and rotate; domain separation over the already-required `secret_key` reaches the same isolation.
- **Signing with `secret_key` directly (no derived subkey).** Rejected: domain separation keeps share signatures from being confusable with other keyed uses of the same secret.

Consequence: links survive restarts (because `secret_key` must be stable) and are revoked only by rotating `secret_key`, which also logs users out. This is the accepted trade-off from the proposal.

### Decision: Add the `hmac` crate

`sha2` is present but does not provide HMAC. Add `hmac` (RustCrypto) and use `Mac::verify_slice` for constant-time verification rather than hand-rolling HMAC or a byte comparison.

### Decision: A dedicated `/s` public scope, not a weakened middleware

The public handlers are registered on a `web::scope("/s")` that is **not** wrapped by `SessionOrBasicAuth`. Each handler verifies the token itself. `SessionOrBasicAuth` stays exactly as it is for feeds, `/media/**`, and `/images/**`.

Alternatives considered:

- **Teach `SessionOrBasicAuth` to accept a token.** Rejected: it would couple a widely used guard to a single feature and require every wrapped surface to reason about share tokens.
- **Register the token routes inside `/media`.** Rejected: `/media/**` is auth-wrapped; bypassing auth there would create an easily misread hole.

Public routes:

```
POST /api/1.0/episodes/{yt_id}/share/   (under RequireSession)  -> mint
GET  /s/{token}/episode.json            (public, token-gated)   -> metadata
GET  /s/{token}/audio.mp3              (public, token-gated)   -> audio
HEAD /s/{token}/audio.mp3              (public, token-gated)
GET  /app/share/{token}                 (SPA static shell)      -> page
```

Any malformed, forged, or expired token responds `404 Not Found` across the public `GET` routes. Using `404` uniformly avoids distinguishing "never existed" from "expired", and it keeps these routes out of the `401`/`WWW-Authenticate` contract that only applies to the authenticated surfaces.

### Decision: Extract and reuse the media streaming core

`serve_media` currently resolves the relative path, selects the representation, then applies range/`ETag`/`304` logic. Split out a helper that streams an already-resolved on-disk file with those semantics and returns an `HttpResponse`. Both the protected handler and the share handler resolve the file and call it.

Alternatives considered:

- **Call `serve_media` from the share handler with a synthesized `/media`-relative path.** Rejected: it re-resolves inside the protected model and obscures the token authorization boundary.
- **Duplicate the range/`ETag` code.** Rejected: two copies of subtle cache/seek logic will drift.

The share handler resolves the episode by `yt_id`, derives the owning channel slug, selects the current representation (`selected_media`, honoring `sponsorblock_enabled`), and 404s if neither the episode nor its file exists. Public share responses SHALL carry `Cache-Control: private` so a shared proxy cannot serve one visitor's capability response to another, while `ETag`/`Last-Modified` still let the visitor's own browser revalidate.

### Decision: A minimal dedicated metadata shape for the public page

`GET /s/{token}/episode.json` returns only `{ title, channel_title, description, image, duration, published_at, expires_at, audio_url }` using `config.url` for `audio_url` (`{url}/s/{token}/audio.mp3`). It is not wrapped in the JSON API's `CustomResponse` and exposes no `user`, progress, favorite, chapter, or SponsorBlock fields.

Alternatives considered: reusing `CustomResponse` with `user: null` was rejected because it implies an authenticated API surface and couples a throwaway public page to the API response contract. The `image` is the episode's existing remote image URL (the one the feed already publishes), so the page needs no new image route; `/images/**` stays protected.

### Decision: Distinguish guest-only routes from public routes in the SPA

`meta.public` becomes "no session required and do not redirect an authenticated visitor away". The login route moves to `meta.guestOnly: true`, and `router.beforeEach` redirects authenticated users away only from guest-only routes. This lets an authenticated user open a share link without being bounced to `channels`, while preserving the existing login behavior.

Alternatives considered: special-casing the share path inside `beforeEach` was rejected as brittle; keeping the current `meta.public` behavior was rejected because it would break sharing for logged-in users.

### Decision: The share page uses a standalone inline player

`frontend/src/views/ShareView.vue` fetches `/s/{token}/episode.json` and renders a plain `<audio controls>` element bound to the returned `audio_url`. It does not use the global persistent player store, which assumes an authenticated context, an episode payload with progress, and cross-episode queue semantics that a public, stateless page must not touch. On `404` the view renders an unavailable state.

### Decision: `share_ttl_days` config with a default of 30

`share_ttl_days` is optional in `config.yml`, defaults to `30`, and is read at startup. A non-positive value is treated as the default. It is documented in `README.md`.

## Risks / Trade-offs

- **Bearer capability leakage** (chat, logs, `Referer`) → the token is a path segment, the app already sets `Referrer-Policy: strict-origin-when-cross-origin`, and the handlers must not log the raw token; the 30-day cap bounds exposure.
- **No per-link revocation** → accepted; documented as revoke-by-rotating-`secret_key`.
- **Rotating `secret_key` also signs out users** → accepted trade-off; documented in `README.md` and the proposal.
- **Shared caches could serve capability responses** → public share responses set `Cache-Control: private`.
- **Deleted or missing episode/media with a still-valid token** → the metadata and audio handlers respond `404`, matching the spec.
- **Authenticated-user routing regression** → covered by moving login to `meta.guestOnly` and adding router/component tests so `/login` still bounces authenticated users while `/share/:token` does not.
- **`exp` boundary/clock skew** → compare against server time; a token is valid while `now <= exp`, which is deterministic on the single server.

## Migration Plan

- No database migration.
- Add `hmac` to `Cargo.toml`; add optional `share_ttl_days` to `config.yml` and its README row (default preserves 30 days when omitted).
- Deploy: existing protected behavior is unchanged, so rollout is additive; share links minted on the new build are self-contained and survive restarts.
- Rollback: remove the `/s` scope and the mint route; already-shared links stop resolving. No data to clean up because there is no share state.

## Open Questions

- Whether to display the link's expiry date on the public page is a cosmetic, deferrable choice; it does not affect the specs, approach, or task breakdown.
