## Context

See `proposal.md` for motivation. The current implementation has three relevant properties:

- `src/handlers/media.rs` resolves `/media/{path:.*}` purely as a filesystem path under the audios directory. It has no database access and maps the URL segment directly to a file.
- `src/handlers/feed.rs` builds the `<enclosure>` URL from the selected physical filename: `{yt_id}.mp3`, or `{yt_id}.sponsorblock.{hash}.mp3` when a processed derivative is active.
- `frontend/src/stores/player.ts` loads `/media/{slug}/{yt_id}.mp3` as the original source and relies on the SponsorBlock segment payload to skip rejected intervals client-side, so it must keep receiving the original audio.

Processed derivatives remain `{yt_id}.sponsorblock.{processing-hash-prefix}.mp3` on disk, and the original remains `{yt_id}.mp3`. The SponsorBlock cache row (`sponsorblock_cache`) holds the active `processed_filename` and `processed_duration` per episode.

## Goals / Non-Goals

**Goals:**

- Make the feed `<enclosure>` URL stable per episode so clients that pin the URL on first sight (e.g. PodcastAddict keyed by GUID) still resolve the current audio.
- Resolve the stable URL to the processed audio when SponsorBlock is enabled and an active derivative exists, otherwise to the original.
- Keep the web player on the original timeline so client-side skipping and progress math are unchanged.
- Preserve every previously published URL (`{yt_id}.sponsorblock.{hash}.mp3`) and the original `{yt_id}.mp3` bytes.

**Non-Goals:**

- No change to how derivatives are generated, named, or reconciled.
- No database schema or migration change.
- No change to authentication, byte-range semantics, or conditional-request behavior beyond deriving validators from the served file.
- No change to episode API payloads.

## Decisions

### Decision: Stable feed URL plus a dedicated original route, rather than a GUID change

The feed publishes `{url}/media/{slug}/{yt_id}.mp3` unconditionally. The same URL is resolved server-side to the active processed file when eligible. The web player switches to `/media/{slug}/{yt_id}.original.mp3`, which always maps to the on-disk original `{yt_id}.mp3`.

Alternatives considered:

- **Version the `<guid>` with the processing hash.** Rejected: it makes podcast clients surface a separate episode for every processing change, duplicating the episode and fragmenting playback progress.
- **Serve the processed file at `{yt_id}.mp3` and leave the player on the same URL.** Rejected: the player would receive processed audio while still applying client-side seeks computed on the original timeline, corrupting skipping and progress.
- **Keep the current hash-versioned enclosure URL.** Rejected: it does not reach clients that pin the enclosure URL by GUID, which is the reported failure.

### Decision: Media resolution consults the database only for the plain `{yt_id}.mp3` form

`serve_media` will take `Data<AppState>` and, after the existing path-traversal guard, classify the request:

- `{slug}/{yt_id}.original.mp3` → serve `{slug}/{yt_id}.mp3` (no database lookup).
- `{slug}/{yt_id}.mp3` (filename ends in `.mp3`, contains no `.sponsorblock.`, not `.original.mp3`) and `sponsorblock_enabled` → look up the episode by channel slug and `yt_id`; if `processed_filename` is set and the file exists under the channel directory, serve that file, else serve `{yt_id}.mp3`.
- Anything else (including `{yt_id}.sponsorblock.{hash}.mp3`) → serve as today.

The `.original` suffix is safe from collision: YouTube ids are fixed-length tokens without dots, so `{yt_id}.original` can never be a real id. It also cannot collide with a derivative, whose marker is `.sponsorblock.`.

Alternatives considered:

- **Resolve by rewriting the file on disk or symlinking.** Rejected: it mutates the original/derivative layout that retention and orphan cleanup already understand.
- **Cache the resolution in memory.** Deferred: the plain `.mp3` URL is only hit by feed downloads and cached-feed clients, not by the interactive player; a single indexed lookup per request is acceptable, and correctness stays simple. A cache can be added later without a spec change.

### Decision: Compute the enclosure `length` from the served physical file

`feed.rs` already resolves the selected physical file (processed when active) and reports its byte size. With the stable URL, that same physical file is what the stable URL serves, so the existing `length` computation stays correct. `itunes:duration` continues to use the selected representation's duration.

### Decision: Chapters stay keyed to the served representation internally

`get_chapters` already compares the selected physical filename against the active `processed_filename` to decide whether to translate chapters. That internal check is unaffected by the stable URL; only the spec wording changes from "selected feed enclosure" to "served representation".

## Risks / Trade-offs

- **[Risk] Content at a stable URL changes when the active derivative changes.** A client or cache holding `{yt_id}.mp3` may serve stale bytes. → Mitigation: `ETag`, `Last-Modified`, `Content-Length`, range, and conditional responses are already derived from the physical file served, so revalidation sees the change. Podcast clients re-download only on demand, which is the intended trigger.
- **[Risk] A database lookup on the media hot path.** → Mitigation: only the plain `{yt_id}.mp3` form is resolved through the database; the interactive player uses `.original.mp3` and never triggers it. Episodes are already indexed by `(channel_id, yt_id)` and channels by slug.
- **[Risk] Existing subscribers keep their already-downloaded original file until they re-download.** → Accepted: the stable URL makes the manual re-download path return the processed audio; auto-updating existing downloads is not something the server can force without changing episode identity.
- **[Risk] Old cached feeds still point at `{yt_id}.sponsorblock.{hash}.mp3`.** → Mitigation: those routes remain available and unchanged.
- **[Trade-off] The original audio remains reachable to authenticated clients via `.original.mp3`.** → Accepted: the original was already reachable before this change; access control is unchanged.

## Migration Plan

- Deploy backend and frontend together. No database migration and no file rename.
- Existing subscriptions: their stored `{yt_id}.mp3` URL now serves the processed derivative when one is active, so a re-download retrieves trimmed audio. Their existing downloaded file is untouched until re-downloaded.
- Rollback: revert the deployment. Clients that fetched the stable URL while the new backend was live have already downloaded processed audio; reverting only restores feed behavior and does not corrupt files.

## Open Questions

None.
