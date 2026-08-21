## Context

Today `channel.image` stores the remote YouTube URL (`yt3.googleusercontent.com`) and the frontend hotlinks it directly on every page render. The worker's per-channel update never touches the image; images are only fetched at creation and via the manual `refresh_image` endpoint. `limit-youtube-concurrency` (separate change) will serialize all YouTube traffic; this change removes the browser-side image traffic and ties image refresh to channel updates.

## Goals / Non-Goals

**Goals:**
- Local, stable, per-channel image URL served by the app.
- Image cache refreshed on create, manual refresh, and every channel sync.
- No YouTube connections from page rendering.

**Non-Goals:**
- No thumbnail regeneration/resizing (store original bytes).
- No caching of arbitrary remote content (only channel covers).
- No frontend redesign (the `image` field just becomes local).

## Decisions

- **Cache directory `images/` with per-slug files (`{slug}.jpg`) plus a static route** mirroring the existing `audios_dir()`/`html_path` pattern (helper fn choosing `/app/images` vs local `images/`). The static route serves only the cache directory (actix `Files::new` scoped to it), which satisfies the "no arbitrary paths" requirement.
- **`image` field semantics change:** API returns the local URL (`/images/{slug}.jpg`); the remote URL is no longer needed by clients. On transitions (a channel with old remote URL and no cached file yet), the first successful fetch populates the file; until then `image` stays as-is/empty per the spec's graceful-degradation requirement.
- **Refresh points:** hook the image download into the same code path that fetches YTInfo metadata. Concretely:
  - `Channel::new` (creation): after `YTInfo::new`, download `ogs:image` bytes → cache.
  - `Channel::update_image` (manual refresh): same download → replaces file.
  - worker `update_channel` (sync): fetch fresh metadata from `channel.url` and refresh the cached file, mirroring `update_image` semantics while keeping the channel's DB fields otherwise unchanged. This is the new "refresh on every channel update" behavior.
- **Download is a bounded HTTP GET** through the existing blocking-fetch pattern (see `fix-blocking-io-in-handlers`): reuse/spawn the same style of fetch; it runs inside `YTInfo::new`'s fn or a sibling helper so the single-connection throttle (`limit-youtube-concurrency`, not yet implemented) will automatically cover it once that change lands. Until then, a bounded fetch with a timeout keeps behavior safe.
- **Write atomically:** download to a temp file then `rename` over the destination so a concurrent read never sees a half-written file; on failure, keep the old file and log, without touching the channel row.

## Risks / Trade-offs

- [Disk growth (one file per channel, unbounded size)] → Mitigated: covers are small JPEGs; fail the write if the body exceeds a sane cap (e.g. 5 MB).
- [Stale cached image between syncs] → Accepted by design: refresh cadence = every channel update; matches user requirement.
- [Serving images unauthenticated] → Public covers are already hotlinked from YouTube today; local serving exposes no more than the old behavior, and the route is confined to the cache directory.
- [Interaction with the throttle change] → The shared fetch path means image downloads automatically respect the single-slot policy once `limit-youtube-concurrency` is implemented; this change must not bypass it.

## Migration Plan

1. Add cache dir helper + static route + install-time creation of the directory.
2. Populate cache on create/manual refresh/sync (with atomic writes + size cap).
3. Frontend keeps rendering `channel.image` (now local). No UI change required.
4. Optional cleanup: once hotlinking stops, the yt3 CORS allowlist entry can be removed in a follow-up.
5. Rollback: disable the route and reverse the `image` field semantics; remote URLs return as before.

## Open Questions

- Whether the cache refresh during worker sync should also run when the channel is inactive (`active=false`); proposal lean: skip it, consistent with the `active` flag change.