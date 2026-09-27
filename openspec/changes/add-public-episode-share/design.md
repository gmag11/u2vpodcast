# Design

## Context

See `proposal.md` - Why. Relevant current state:

- `src/handlers/mod.rs` wraps `/media/**`, `/feed.xml`, `/channels/{slug}/feed.xml`, `/{slug}/feed.xml`, and `/images/**` with `SessionOrBasicAuth`, and the JSON API under `/api/1.0/*` with `RequireSession`. `SessionOrBasicAuth` is gated by `config.with_authentication`.
- `src/handlers/media.rs` already implements byte-range, `ETag`, `Last-Modified`, conditional `304`, and the SponsorBlock/original representation selection inside `serve_media`.
- `src/models/config.rs` loads and validates `config.yml`. `secret_key` is required, validated to at least 64 bytes, and stable across restarts (it signs session cookies). `episodes.webpage_url` holds the episode's canonical YouTube URI and is available on the `Episode` model (`read_by_yt_id_with_channel`).
- The SPA is served as static files under `/app`; `frontend/src/router/index.ts` currently treats `meta.public` as "guest-only".

## Goals / Non-Goals

**Goals:**

- Authorize a public share surface with a deterministic, permanent, self-contained token and no new persistent state.
- Reuse the existing media streaming and representation-selection behavior so range/seek/`ETag` semantics are identical to protected playback.
- Keep the existing protected surfaces and `with_authentication` semantics untouched.

**Non-Goals:**

- Per-link revocation, audit trail, or analytics (only rotating the share secret revokes).
- A time-based or file-based expiry.
- An RSS feed or any podcast-client surface for a shared episode.
- Persisting the anonymous visitor's playback progress.

## Decisions

### Decision: Permanent token = `{yt_id}-{hex(HMAC(share_secret, "v1|{webpage_url}"))}`

```
hash  = HMAC-SHA256(effective_share_secret, "u2vpodcast/share/v1|" + episode.webpage_url)
token = "{yt_id}-{lowercase_hex(hash)}"
URL   = "{config.url}/app/share/{token}"
```

Verification reads the `{yt_id}` from the token, loads the episode (O(1) by `yt_id`), recomputes the hash over its `webpage_url`, and compares in constant time (`Mac::verify_slice`). The token is deterministic — the same episode and secret always yield the same token — and carries no expiry.

Alternatives considered:

- **Token = only the hash of the URI.** Rejected: resolving it needs a `share_hash` column (migration) or a full scan of episodes per request. Embedding `yt_id` keeps lookup O(1) without state; `yt_id` is already public, and the hash remains the unforgeable capability.
- **Hash over `yt_id` instead of `webpage_url`.** Rejected: the user asked for a hash of the episode URI; hashing `webpage_url` ties the link to the episode's canonical address.
- **A separate `share_links` table with opaque tokens.** Rejected: adds a migration and state for a value that is cheaply re-derivable.
- **A time-based expiry.** Rejected: episodes are already removed by retention, so permanence is preferable and simpler.

Consequence: links never expire. A link stops working only when its episode is deleted (retention) or the effective share secret is rotated.

### Decision: `share_secret` config, falling back to `secret_key`

Add an optional `share_secret` to `Config`. When it is absent or empty, the effective share secret is `secret_key`. A dedicated secret lets an operator rotate public links without invalidating session cookies. The `hmac` crate provides constant-time verification.

### Decision: A dedicated `/s` public scope, not a weakened middleware

The public handlers are registered on a `web::scope("/s")` that is **not** wrapped by `SessionOrBasicAuth`; each handler verifies the token itself. Public routes:

```
POST /api/1.0/episodes/{yt_id}/share/   (under RequireSession)  -> mint
GET  /s/{token}/episode.json            (public, token-gated)   -> metadata
GET  /s/{token}/audio.mp3              (public, token-gated)   -> audio
HEAD /s/{token}/audio.mp3              (public, token-gated)
GET  /app/share/{token}                 (SPA static shell)      -> page
```

Any malformed, forged, or unresolvable token responds `404 Not Found` uniformly, avoiding a `401`/`WWW-Authenticate` contract that only applies to authenticated surfaces.

### Decision: Extract and reuse the media streaming core

Split the range/`ETag`/`304`/HEAD logic out of `serve_media` into a helper that streams an already-resolved on-disk file, so both the protected handler and the share handler select the representation and then stream through it. Public share responses SHALL carry `Cache-Control: private` so a shared proxy cannot serve one visitor's capability response to another, while `ETag`/`Last-Modified` still let the visitor's own browser revalidate.

### Decision: A minimal dedicated metadata shape for the public page

`GET /s/{token}/episode.json` returns only `{ title, channel_title, description, image, duration, published_at, audio_url }` using `config.url` for `audio_url` (`{url}/s/{token}/audio.mp3`). It is not wrapped in the JSON API's `CustomResponse`, exposes no `user`, progress, favorite, chapter, or SponsorBlock fields, and carries no expiry. The `image` is the episode's existing remote image URL, so no new image route is needed and `/images/**` stays protected.

### Decision: Distinguish guest-only routes from public routes in the SPA

`meta.public` becomes "no session required and do not redirect an authenticated visitor away". The login route moves to `meta.guestOnly: true`, and `router.beforeEach` redirects authenticated users away only from guest-only routes. This lets an authenticated user open a share link without being bounced to `channels`.

### Decision: The share page uses a standalone inline player

`frontend/src/views/ShareView.vue` fetches `/s/{token}/episode.json` and renders a plain `<audio controls>` element bound to the returned `audio_url`. It does not use the global persistent player store, which assumes an authenticated context and cross-episode queue semantics a public, stateless page must not touch. On `404` the view renders an unavailable state.

## Risks / Trade-offs

- **Permanent bearer capability** → a leaked link works forever. Mitigation: the hash is unguessable, the feed is protected, tokens are not logged, and rotating `share_secret` revokes every link without touching sessions.
- **Rotating `share_secret` also breaks every outstanding link** → intended; that is the only revocation mechanism.
- **A renamed/changed `webpage_url` invalidates a link** → `webpage_url` is the episode's stable canonical URI, so this is acceptable and matches "hash of the episode URI".
- **Shared caches could serve capability responses** → public share responses set `Cache-Control: private`.
- **Deleted episode/media with a still-valid token** → the metadata and audio handlers respond `404`.
- **Authenticated-user routing regression** → covered by moving login to `meta.guestOnly` and adding router/component tests.

## Migration Plan

- No database migration.
- Add optional `share_secret` to `config.yml` and its README row (falling back to `secret_key` when omitted).
- Deploy: existing protected behavior is unchanged, so rollout is additive; links are deterministic and survive restarts.
- Rollback: remove the `/s` scope and the mint route; already-shared links stop resolving. No data to clean up because there is no share state.

## Open Questions

None that affect the specs, approach, or tasks.
