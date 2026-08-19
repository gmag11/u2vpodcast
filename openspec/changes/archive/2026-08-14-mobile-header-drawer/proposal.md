## Why

On mobile (<768px) the header overflows: brand, inline nav links (Channels, History), per-view action buttons, theme toggle, and session controls all render in the fixed `h-20` bar, pushing items off screen. The mobile layout was never designed — the inline nav added by the history screen has no responsive hiding, and the header search slot is already hidden on small screens, leaving the Channels page without search on mobile.

## What Changes

- On mobile, the header brand renders as the icon only; the "U2V Podcast" wordmark is hidden.
- On mobile, the inline nav links (Channels, History) are removed from the bar and relocated into a right-side drawer opened by a hamburger button. The drawer shows the logged-in user (avatar + name), the nav links, and Logout, and closes on backdrop click, Escape, route change, or item selection.
- On desktop (`md+`) the current header layout is unchanged.
- Per-view header actions (Create New on Channels, Refresh on Episodes) render as icon-only buttons on mobile, keeping their text on desktop.
- The theme toggle remains visible in the mobile bar.
- A mobile search toggle appears on views that provide a header search slot (Channels): it expands a full-width search input row below the header on mobile. This restores channel search on mobile, which is currently unreachable.

## Capabilities

### New Capabilities
- `mobile-header-drawer`: mobile-specific reorganization of the app header — icon-only branding, a right-side navigation/session drawer, icon-only per-view actions, and an expandable mobile search row.

### Modified Capabilities
- None.

## Impact

- **Code**:
  - `frontend/src/components/AppHeader.vue`: responsive breakpoints for brand/nav/actions, hamburger button, side drawer, mobile search toggle and expandable search row.
  - `frontend/src/views/ChannelsView.vue` and `frontend/src/views/EpisodesView.vue`: action button labels become responsive (text hidden on mobile, icon-only).
  - Component tests for the header drawer behavior.
- **APIs**: none.
- **Dependencies**: none (existing Vue Router, Pinia, `@phosphor-icons/vue`).
- **DB**: none.
- **Frontend**: header component and the two views that provide action buttons; desktop rendering unchanged.
