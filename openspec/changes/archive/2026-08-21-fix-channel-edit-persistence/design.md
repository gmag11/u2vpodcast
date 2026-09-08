## Context

`UpdateChannel` (`src/models/channel.rs`) carries `id, url, active, first, max` — no `title`. `Channel::update` SQL writes only `active, first, max, updated_at`. The edit dialog (`AddChannelDialog.vue`) sends `title` (dropped by serde before the model is even built) and `url` (accepted but never written), and `ChannelsView.saveChannel` replaces the local row optimistically.

## Goals / Non-Goals

**Goals:**
- Title and URL edits actually persist and survive reload.
- Slug and audio directory remain immutable.

**Non-Goals:**
- No slug regeneration, no feed URL changes, no UI redesign.
- No change to `description`/`image` edit behavior beyond what already persists (only if trivially safe).

## Decisions

- **Extend `UpdateChannel` with `title: String` and persist title + url in `Channel::update`.** Minimal, keeps the response shape (channel) identical in structure.
- **Do NOT re-slugify on update.** Slug immutability is an existing spec contract (`channel-slugs`); renaming must never move the audio directory or break feed/media URLs.
- **Drive the UI from server truth:** `saveChannel` should apply the `PUT` response (`result.data`) to the local row instead of the raw local object, so nothing the server rejected stays visible. If the server returns the full channel, this is a one-line change.
- **Reject empty titles** in the handler/model with a clear error rather than storing blanks that would break slug/feed formatting.

## Risks / Trade-offs

- [If `first`/`max` semantics ride on edit payloads, validating title must not disturb them] → Mitigated: only title/url fields are added to the SQL; existing field handling untouched.
- [Empty-title guard could break a current flow if the dialog allows blank] → The dialog sends `title || ''`; the backend guard turns that into a visible error instead of silent loss.

## Migration Plan

Code change; no data migration. Existing channels update in place.

## Open Questions

None.