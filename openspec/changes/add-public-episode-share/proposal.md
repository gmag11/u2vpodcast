# Proposal

## Why

This deployment is protected by a single admin credential and, by default, `with_authentication: true` guards every RSS feed and `/media/**`. An operator who wants to share one episode with someone outside the deployment has no way to do it short of handing over credentials or turning the whole server public via the global `with_authentication` switch. A per-episode, unguessable public link that plays in a browser lets an authenticated user share a single episode without exposing the rest of the library.

## What Changes

- Add a permanent **share token** bound to one episode: `{yt_id}-{hash}`, where `{hash}` is the lowercase-hex HMAC-SHA256 of the episode's canonical URI (`webpage_url`) under a configurable share secret with a fixed domain-separation prefix. It is deterministic (the same episode and secret always produce the same token), has no expiry, and needs no database state.
- Add an authenticated endpoint that returns the public share URL for one episode (no expiry in the response).
- Add public, unauthenticated routes that accept a valid token only:
  - `GET /s/{token}/episode.json` — minimal metadata for the public page (title, channel title, description, image, duration, publish date; no expiry and no user, progress, or favorite data).
  - `GET|HEAD /s/{token}/audio.mp3` — the episode's selected audio (SponsorBlock-processed when enabled, otherwise the original) reusing the existing `Range`, `ETag`, and `304` semantics.
- Add a public share page in the SPA at `/app/share/{token}` that renders the episode and plays it inline, without a session and without redirecting an already-authenticated visitor away.
- Add a share action on episode cards for authenticated users that copies the link.
- Configuration: add `share_secret` (falls back to `secret_key` when absent); there is no time-based expiry to configure.
- Revocation: rotating the effective share secret invalidates every outstanding link. There is no per-link revocation, and links do not expire; a link stops working only when its episode is deleted or the secret is rotated.

Non-goals: an RSS feed for a single shared episode; per-channel public sharing; a share-links table or per-link revocation; sharing playlists; changing `with_authentication` semantics; any file- or time-based expiry.

## Capabilities

### New Capabilities
- `episode-sharing`: minting permanent signed per-episode share links, the public tokenized metadata and audio routes, and the public share page.

### Modified Capabilities
- `route-protection`: the public share routes are explicitly exempt from the session/Basic-Auth guard and are instead authorized by a signed token.

## Impact

- Code: `src/models/config.rs` (`share_secret`), `src/utils/share.rs` (deterministic token), `src/handlers/share.rs` (mint and public handlers), `src/handlers/mod.rs` (route registration), `src/handlers/media.rs` (reuse the streaming core), `frontend/src/router/index.ts`, `frontend/src/views/ShareView.vue`, `frontend/src/lib/api/client.ts`, `frontend/src/types.ts`, `frontend/src/components/EpisodeCard.vue`, and `frontend/src/i18n/locales/{en,es}.json`.
- Config: `config.yml` gains optional `share_secret`; `README.md` documents it.
- No database migration, no new dependency, no change to existing protected surfaces, and no change to `with_authentication` behavior.
- Security: a share link is a permanent bearer capability. Anyone holding it can read that episode's metadata and audio until the episode is deleted or the share secret is rotated.
