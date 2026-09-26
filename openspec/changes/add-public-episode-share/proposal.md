# Proposal

## Why

This deployment is protected by a single admin credential and, by default, `with_authentication: true` guards every RSS feed and `/media/**`. An operator who wants to share one episode with someone outside the deployment has no way to do it short of handing over credentials or turning the whole server public via the global `with_authentication` switch. A per-episode, unguessable, expiring public link lets an authenticated user share a single episode that plays in a browser without exposing the rest of the library.

## What Changes

- Add a stateless, HMAC-signed **share token** bound to one episode (`yt_id`) with an expiry (default 30 days). The signing key is derived from the existing `secret_key` with domain separation; no new database table or per-link state is introduced.
- Add an authenticated endpoint that mints a share link for one episode and returns its public URL.
- Add public, unauthenticated routes that accept a valid token only:
  - `GET /s/{token}/episode.json` — minimal metadata for the public page (title, channel title, description, image, duration; no user, progress, or favorite data).
  - `GET|HEAD /s/{token}/audio.mp3` — the episode's selected audio (SponsorBlock-processed when enabled, otherwise the original) reusing the existing `Range`, `ETag`, and `304` semantics.
- Add a public share page in the SPA at `/app/share/{token}` that renders the episode and plays it inline, without a session and without redirecting an already-authenticated visitor away.
- Add a share action on episode cards for authenticated users that mints the link and copies it.
- Add a `share_ttl_days` configuration value (default `30`).
- Revocation: rotating `secret_key` invalidates every outstanding share link (and, as today, every session). There is no per-link revocation.

Non-goals: an RSS feed for a single shared episode; per-channel public sharing; a share-links table or per-link revocation; sharing playlists; changing `with_authentication` semantics.

## Capabilities

### New Capabilities
- `episode-sharing`: minting signed per-episode share links, the public tokenized metadata and audio routes, and the public share page.

### Modified Capabilities
- `route-protection`: the public share routes are explicitly exempt from the session/Basic-Auth guard and are instead authorized by a signed token.

## Impact

- Code: `src/models/config.rs` (`share_ttl_days`), a new `src/handlers/share.rs` (mint endpoint and public handlers), `src/handlers/mod.rs` (route registration), `src/utils/` (token sign/verify), `src/handlers/media.rs` (extract the streaming core for reuse), `frontend/src/router/index.ts` (public route), a new `frontend/src/views/ShareView.vue`, `frontend/src/lib/api/client.ts`, `frontend/src/components/EpisodeCard.vue`, and `frontend/src/i18n/locales/{en,es}.json`.
- Dependencies: add the `hmac` crate for constant-time HMAC verification (`sha2` is already present).
- Config: `config.yml` gains optional `share_ttl_days` (default 30); `README.md` documents it.
- No database migration, no change to existing protected surfaces, and no change to `with_authentication` behavior.
- Security: a share link is a bearer capability. Anyone holding it can read that episode's metadata and audio until it expires; rotating `secret_key` is the only revocation mechanism.
