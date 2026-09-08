## Why

Editing a channel shows editable Title and Url inputs, but the backend discards both: `UpdateChannel` does not even contain `title`, and `Channel::update`'s SQL only writes `active, first, max, updated_at`. The frontend optimistically mutates its local array, so edits look saved and silently vanish on reload — misleading users into believing derivative artifacts (feeds, slugs, cover names) were updated.

## What Changes

- Persist `title` and `url` on channel edit: extend `UpdateChannel` with `title`, update the SQL and response.
- Keep the slug **immutable** (per `channel-slugs` spec): renaming `title` must not change the slug or audio directory.
- Keep the optimistic UI refresh consistent with server truth (the PUT response drives the local row).
- Clarify/restrict image and description fields per what the backend actually stores.

## Capabilities

### New Capabilities

- `channel-editing`: Defines that editing a channel persists the edited metadata without renaming its slug.

### Modified Capabilities

(none)

## Impact

- `src/models/channel.rs` (`UpdateChannel`, `Channel::update` SQL), `src/handlers/channels.rs`, `frontend/src/components/AddChannelDialog.vue`, `frontend/src/views/ChannelsView.vue`.
- Slug/audio directory naming unchanged (explicit non-goal).
- Regression guard: re-analysis against `docs/bug-review-2026-08-21.md`; no new bugs — in particular, validating title changes must not create duplicate/broken slugs and must not corrupt `first`/`max` semantics.