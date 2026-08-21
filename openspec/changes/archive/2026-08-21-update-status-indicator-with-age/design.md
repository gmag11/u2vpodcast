## Context

The channel card (`frontend/src/components/ChannelCard.vue`) currently renders two unrelated indicators:

- A green/red sync status dot at the top-left, driven by `channel.last_sync_ok`, with a `title` attribute ("Last sync succeeded" / "Last sync failed").
- A last-sync age badge at the bottom-left, driven by `channel.last_sync_at` via `lastSyncAge()` (`frontend/src/lib/utils/channel.sync.age.ts`), with a `title` attribute showing the raw timestamp.

Users must scan two corners of the card to correlate sync health with recency. The card already uses a CSS group-hover tooltip pattern (`.group` wrapper + absolutely positioned span) for the action buttons.

## Goals / Non-Goals

**Goals:**

- One bottom-left group containing the status dot (left) and age badge (right).
- Single shared tooltip on the group: `Updated <age> ago. Status: Ok|Error`.
- Remove the standalone top-left dot and per-element tooltips.

**Non-Goals:**

- No changes to sync recording, API payloads, or `lastSyncAge()` formatting.
- No new tooltip library; reuse the existing group-hover CSS pattern.
- No changes to other card badges (last-episode age top-right stays as is).

## Decisions

- **Reuse the existing `.group` + hover tooltip pattern.** Wrap dot and badge in one `div.group.relative`; render one tooltip span inside it. Rationale: consistent with the rest of the card, zero dependencies.
  - Alternative: a shared tooltip component — overkill for one usage today.
- **Tooltip text built in a computed property** (`syncStatusTooltip`) combining `syncAgeLabel` and `last_sync_ok`, e.g. `` `Updated ${syncAgeLabel.value} ago. Status: Ok` ``. Rationale: keeps template terse and testable.
- **Visibility rules:** show the group when either `syncAgeLabel` is non-empty or `last_sync_ok === true/false`. The dot renders only when a success flag exists; the badge only when `syncAgeLabel` is non-empty. This preserves current never-synced behavior (nothing rendered).
- **Drop both `title` attributes** on dot and badge so hover shows only the shared tooltip (avoids double tooltips from native title).

## Risks / Trade-offs

- [Hover-only feedback is lost on touch devices] → Same trade-off as existing action-button tooltips; acceptable status quo.
- [`Updated 0h ago` reads slightly odd for fresh syncs] → Matches the truncated-age convention already shipped; not changing formatter.
- [Existing specs describe the dot at top-left] → Covered by delta specs in this change (`channel-sync-status`, `channel-card-sync-age`).

## Migration Plan

Single-component frontend change. No data migration. Rollback = revert the commit.

## Open Questions

None.
