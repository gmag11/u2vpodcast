## Context

The Vue 3 SPA renders channel cards in `ChannelCard.vue`. Each card already shows a sync-status indicator dot (top-left, from `last_sync_ok`) and a last-episode age badge (top-right, from `last_date` via the `lastEpisodeAge()` helper in `frontend/src/lib/utils/channel.age.ts`). The `Channel` type already exposes `last_sync_at: string | null`, which is recorded by the backend and included in every channel payload (see capability `channel-sync-status`). There is currently no visual indication of how stale a channel's local data is.

This change adds a small non-interactive badge to the bottom-left corner of each channel card showing the truncated hours (and larger units) since the last sync.

## Goals / Non-Goals

**Goals:**
- Surface staleness at a glance by showing truncated hours since the last sync.
- Reuse the same visual language as the existing age badge for consistency.
- Pure frontend change; no backend or API changes.

**Non-Goals:**
- No changes to the sync-status dot (top-left) or the last-episode age badge (top-right).
- No new styling system, design tokens, or dependencies.
- No automatic refresh or syncing behavior changes.

## Decisions

### 1. New helper `lastSyncAge()` mirroring `lastEpisodeAge()`
- **Decision**: Add `frontend/src/lib/utils/channel.sync.age.ts` exporting `lastSyncAge(lastSyncAt: string | null): string`, structured exactly like `lastEpisodeAge()` but starting with hours.
- **Why**: Keeps the age-formatting logic testable in isolation (same pattern as the existing unit-tested helper) and keeps the component template clean. Alternative considered: inline the math in the component, but that duplicates the display logic and is not unit-testable in isolation.
- **Alternatives**: Inline computed in `ChannelCard.vue` — rejected for testability and consistency.

### 2. Unit scale: hours → days → weeks → months → years
- **Decision**: Use truncated units — `Nh` under 24h, `Nd` under 7d, `Nw` under 30d, `Nm` under 365d, `Ny` at 365d+. Sub-hour truncates to `0h`.
- **Why**: Mirrors the existing `lastEpisodeAge()` conventions (truncation down, compact suffix) so both badges look and behave consistently. The user explicitly requested hours as the primary unit.
- **Alternatives**: Always show hours (e.g. `100h`) — rejected as poor UX for long-unsynced channels; showing minutes for sub-hour ages — rejected to keep the single-unit convention simple and consistent with the sibling badge (`0d`).

### 3. Placement: absolute bottom-left, non-interactive
- **Decision**: Render the badge as an absolutely-positioned span in the bottom-left corner of the card (`absolute bottom-4 left-4`), styled like the existing top-right age badge, non-interactive (no click handler), using the same `z-10` stacking so it never overlaps content.
- **Why**: The user requested bottom-left; the corners top-left (dot) and top-right (age badge) are already occupied. Non-interactive matches the read-only nature of the data.
- **Alternatives**: Tooltip showing precise time — out of scope (non-goal).

### 4. No backend/API change
- **Decision**: Consume the existing `last_sync_at` field already present in the `Channel` type and API payload.
- **Why**: Nothing new needs to be persisted or served; avoids unnecessary churn.

## Risks / Trade-offs

- **Badge clutter / overlap** → Mitigation: place it in the bottom-left (unused corner), reuse the exact pill styling of the age badge, and keep it non-interactive with a small `z-10`.
- **Timezone/clock skew between browser and server** → Mitigation: the age is computed client-side from the ISO `last_sync_at` timestamp; minor skew only affects the shown unit boundary, which is acceptable for a display-only badge.
- **`0h` appearing for a just-synced channel** → Mitigation: intentional and consistent with the existing `0d` behavior of the last-episode age badge; still communicates "just updated".

## Migration Plan

- Deploy the frontend build (`pnpm run build`) together with the rest of the SPA; no DB or backend migration required. Rollback is a normal front-end redeploy.

## Open Questions

- None. The behavior is fully specified by the `channel-card-sync-age` spec.
