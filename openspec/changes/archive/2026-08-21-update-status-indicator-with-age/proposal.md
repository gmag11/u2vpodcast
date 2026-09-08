## Why

On the channel card, the sync status dot (green/red) sits in the top-left corner while the update age badge sits in the bottom-left corner, so users must look at two separate places to understand sync health. Grouping them in one spot with a single combined tooltip communicates both facts at a glance.

## What Changes

- Move the sync status dot (green/red, from `channel.last_sync_ok`) from the top-left of the channel card to the bottom-left corner, immediately left of the existing update age badge.
- Wrap the dot + age badge into a single hover group.
- Replace the individual tooltips with one shared tooltip on the group with format:
  - `Updated 2h ago. Status: Ok` (when `last_sync_ok === true`)
  - `Updated 3h ago. Status: Error` (when `last_sync_ok === false`)
- Remove the old top-left status dot and its `title` attribute.

## Capabilities

### New Capabilities

(none)

### Modified Capabilities

- `channel-sync-status`: The status indicator no longer renders standalone at the top-left; it is part of a bottom-left group next to the update age, and its feedback is delivered via the group tooltip.
- `channel-card-sync-age`: The age badge now shares a tooltip with the status dot ("Updated <age> ago. Status: Ok|Error") instead of showing only the raw last sync timestamp.

## Impact

- Frontend only: `frontend/src/components/ChannelCard.vue`.
- No backend, API, or dependency changes.
- Tests referencing the top-left dot or the age-badge `title` may need updates.
