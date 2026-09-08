## Context

Today `channel.image` stores the remote YouTube URL (`yt3.googleusercontent.com`) and the frontend hotlinks it directly on every page render. The worker's per-channel update never touches the image; images are only fetched at creation and via the manual `refresh_image` endpoint. `limit-youtube-concurrency` (separate change) will serialize all YouTube traffic; this change removes the browser-side image traffic, ties image refresh to channel updates, keeps the cache inside an existing Docker volume, and — per explicit requirement — does **not** serve it publicly.

Docker topology (from `docker-compose.yml` / `Dockerfile`): the image mounts only `audios:/app/audios` and `db:/app/db`; `/app` itself is not a volume. Any new cache path outside those two mounts would be lost on container recreation, so the cache must sit inside one of the existing mounts.

## Goals / Non-Goals

**Goals:**
- Local, stable, per-channel image URL served by the app, **authenticated** like the rest of the API.
- Image cache refreshed on create, manual refresh, and every channel sync — skipping the download when the remote image is unchanged (size probe via `HEAD`).
- Cache persisted inside an existing Docker volume (no new volume in `docker-compose.yml`).
- No YouTube connections from page rendering.

**Non-Goals:**
- No thumbnail regeneration/resizing (store original bytes).
- No caching of arbitrary remote content (only channel covers).
- No frontend redesign (the `image` field just becomes local).
- No new Docker volumes or mounts.
- No checklist/etag-pair exact comparison beyond the size probe (extendable later via stored `ETag`/`Last-Modified`).

## Decisions

- **Cache directory inside the `db` volume:** `{images_dir()}` resolves to `/app/db/images` in the container (already persisted by the `db` volume) and to a local relative `images` directory in development. The `audios` volume was rejected for the cache: `audios/{slug}/` is the audio storage scanned and renamed by the slug migration and per-channel logic, and a channel whose slug is literally `images` would collide with a top-level cache directory. The `db` mount has no such naming constraint and is already required to survive container recreation.
- **Authenticated route:** the cache is exposed through a route wrapped in the existing `SessionOrBasicAuth` middleware (identical to `/media`). Cookie sessions work because the SPA and the API are same-origin — `<img src="channel.image">` carries the session cookie automatically. Unauthenticated fetches (private browsing, fresh cookie) receive `401`. The feeds already require credentials, so podcast clients providing auth for the feed keep working for the cover; clients that cannot send credentials will not load the cover (documented trade-off requested by the operator).
- **Skip unchanged images via `HEAD`:** before a full download, send an HTTP `HEAD` request to the stored remote image URL and read `Content-Length`. If a cached file exists and its on-disk size equals the reported length, the image is unchanged → skip the GET entirely (most sync cycles cost one header request). If `HEAD` errors, times out, or omits `Content-Length`, fall back to the bounded GET. This answers the open question directly: yes, the size can be obtained without downloading the body. (Optional hardening later: store `ETag`/`Last-Modified` from the `HEAD` as a sidecar to strengthen the equality check; out of scope now.)
- **`image` field semantics change:** API returns the local URL (`/images/{slug}.jpg`); the remote URL is no longer needed by clients. On transitions (a channel with old remote URL and no cached file yet), the first successful fetch populates the file; until then `image` stays as-is/empty per the spec's graceful-degradation requirement.
- **Refresh points:** hook the image download into the same code path that fetches YTInfo metadata. Concretely:
  - `Channel::new` (creation): after `YTInfo::new`, run the HEAD-then-GET path → cache.
  - `Channel::update_image` (manual refresh): same path → replaces file.
  - worker `update_channel` (sync): fetch fresh metadata from `channel.url` and refresh the cached file, mirroring `update_image` semantics while keeping the channel's DB fields otherwise unchanged.
  - `Channel::delete`: remove the cached file alongside the audio directory.
- **Probe + download are bounded HTTP requests** through the existing blocking-fetch pattern (see `fix-blocking-io-in-handlers`): reuse/spawn the same style of fetch; it runs inside `YTInfo::new`'s fn or a sibling helper so the single-connection throttle (`limit-youtube-concurrency`, not yet implemented) will automatically cover it once that change lands. Until then, a bounded fetch with a timeout keeps behavior safe.
- **Write atomically:** download to a temp file then `rename` over the destination so a concurrent read never sees a half-written file; on failure, keep the old file and log, without touching the channel row.

## Risks / Trade-offs

- [Disk growth (one file per channel, unbounded size)] → Mitigated: covers are small JPEGs; fail the write if the body exceeds a sane cap (e.g. 5 MB).
- [Stale cached image between syncs] → Accepted by design: refresh cadence = every channel update; unchanged images cost one `HEAD` each cycle.
- [Authenticated images may break clients without credentials] → Explicit operator requirement; same-origin SPA flow carries the cookie, and feed clients already authenticate. Documented.
- [Size-only equality heuristic] → Content-Length can in theory match while bytes differ; rare for static YouTube covers, and the atomic-write + next-cycle refresh self-heals. Optional sidecar `ETag` comparison is the documented extension.
- [Interaction with the throttle change] → The shared fetch path means image downloads (and probes) automatically respect the single-slot policy once `limit-youtube-concurrency` is implemented; this change must not bypass it.
- [Images inside the `db` volume] → Mixes binary files with the SQLite DB in one volume; acceptable (small files, same persistence guarantees, no compose change).

## Migration Plan

1. Add `images_dir()` helper resolving to `/app/db/images` (container) or a local `images` dir; no `docker-compose.yml` change.
2. Register the authenticated `/images` route (wrapped in `SessionOrBasicAuth`) scoped to the cache directory.
3. Populate cache on create/manual refresh/sync with atomic writes + size cap; implement the `HEAD`-vs-local-size skip.
4. Delete the cached file on channel deletion.
5. Frontend keeps rendering `channel.image` (now local, same-origin). No UI change required.
6. Optional cleanup: once hotlinking stops, the yt3 CORS allowlist entry can be removed in a follow-up.
7. Rollback: disable the route and reverse the `image` field semantics; remote URLs return as before.

## Open Questions

- Whether the cache refresh during worker sync should also run when the channel is inactive (`active=false`); proposal lean: skip it, consistent with the `active` flag change.
- Whether to strengthen the equality check with a stored `ETag` sidecar in a follow-up (out of scope here).