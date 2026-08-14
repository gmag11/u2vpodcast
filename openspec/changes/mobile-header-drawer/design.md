## Context

`AppHeader.vue` is the fixed top bar rendered by the three authenticated views (`ChannelsView`, `EpisodesView`, `HistoryView`). It currently mixes: brand (icon + wordmark), inline nav links (Channels, History — added by `add-history-screen` with no responsive hiding), a `#search` slot (Channels only, already `hidden md:flex`), an `#actions` slot (Create New / Refresh), a theme toggle, and the user/logout block. On screens under the `md` breakpoint all of these render in the fixed `h-20` bar and overflow. The mobile header was never designed.

`AppButton.vue` renders the action buttons passed through `#actions`. The `SearchInput` component already provides a magnifier icon and a `modelValue`/`update:modelValue` contract used by the views' search state.

## Goals / Non-Goals

**Goals:**
- Make the header fit mobile widths: icon-only brand, icon-only actions, theme toggle retained.
- Move global navigation and session controls (user + Logout) into a right-side drawer on mobile.
- Give mobile a search affordance on views that provide a `#search` slot (fixes the current missing mobile search on Channels).
- Keep the desktop (`md+`) header byte-for-byte identical to today.

**Non-Goals:**
- Changing navigation architecture or routes.
- Adding a search experience beyond what each view already provides via the `#search` slot.
- Redesigning the desktop header.
- Moving user/session controls on desktop.

## Decisions

### Decision 1: Single responsive component, not a separate mobile header

Keep all logic in `AppHeader.vue` and switch presentation with Tailwind breakpoints. The wordmark gets `hidden lg:inline` (hidden below `lg`, not `md`, so the header fits narrow desktop widths between 768px and 1024px where the full bar would otherwise overflow); the inline nav gets `hidden md:flex`; the hamburger and mobile search toggle get `md:hidden`. A dedicated mobile header component would duplicate slots, auth/theme wiring, and logout handling.

**Why**: the slots (`#brand-icon`, `#search`, `#actions`) are already provided by the views; a second component would force every view to branch on breakpoint. Responsive classes keep one source of truth.

### Decision 2: Right-side drawer owned by AppHeader

Add `drawerOpen` ref. The drawer is a fixed panel (`right-0 top-0 h-full w-72`) under a full-screen backdrop, rendered only when `drawerOpen`. Contents: user avatar + name (from `useAuthStore`), `RouterLink` nav items (Channels, History) with active styling matching the current inline links, and a Logout button. Closes on: backdrop click, Escape (keydown listener), `RouterLink` click, and a `watch` on `route` (so navigation always closes it).

**Why**: keeps all session/nav behavior in the header; the drawer is pure presentation. Right-side placement matches the current user/logout placement on the right and the common mobile pattern.

### Decision 3: Per-view action buttons become icon-only on mobile in the views

`ChannelsView` (Create New) and `EpisodesView` (Refresh) wrap their button label text in `<span class="hidden sm:inline">`, so the `AppButton` renders icon-only on mobile and text+icon on `sm+`. No `AppButton` prop changes needed; the existing `gap-2` layout keeps the icon centered when the label is hidden.

**Why**: the `#actions` slot content is owned by the views; the header cannot restyle slot content. A label wrapper is the smallest change and keeps `AppButton` generic.

### Decision 4: Mobile search via a toggle + expandable row in AppHeader

When `$slots.search` is present, AppHeader renders a magnifier toggle (`md:hidden`) and a mobile search row below the header (absolute, full width, `md:hidden`) shown when `searchOpen`. The same `<slot name="search" />` is also rendered inline for `md+` as today. Both instances bind to the same view state via the existing `v-model` contract.

**Why**: the slot content is a single `SearchInput` bound to the view's `searchQuery`; rendering it in two places is safe because typing in one emits `update:modelValue` to the shared ref and the other reflects the same value. This fixes the current gap where mobile Channels has no search.

### Decision 5: Layering

Header keeps `z-50`; backdrop `z-[60]`; drawer `z-[70]`. Drawer and backdrop must sit above the persistent bottom player. Verify the player's z-index and bump if needed.

**Why**: a fixed bottom player could otherwise intercept taps; explicit z-layering makes the drawer reliably on top.

### Decision 6: Contain channel-card tooltip overflow

The invisible hover tooltips in `ChannelCard` (`absolute whitespace-nowrap`, `opacity-0`) extend past the viewport on mobile and create a horizontal scrollbar. Add `overflow-x-clip` to the Channels page `<main>` so absolute tooltips are painted clipped at the container edge and cannot widen the scrollable area. `overflow-x: clip` (not `hidden`) avoids creating a scroll container and keeps the vertical axis intact.

**Why**: the tooltips must stay `nowrap` to size correctly; clipping at the page container is the smallest change that removes the mobile horizontal scroll without altering the desktop hover behavior.

### Decision 7: Header fits narrow desktop widths

With every item visible (`md`+), the header's min-content width is ~945px, so between 768px and 945px the fixed header overflows its box and right-side items are clipped. The wordmark moves to `lg:inline`, search margins shrink to `mx-4` (back to `mx-8` at `lg`), and the right-group gap becomes `gap-4` until `lg`. Measured with headless Chrome, this reduces the header min-content so it fits from 768px up with no overflow.

**Why**: 768px (tablet portrait / small laptop) is a real device width; hiding just the wordmark and tightening spacing keeps all functionality visible without pushing nav or session controls into the drawer at desktop sizes.

## Risks / Trade-offs

- **[Risk] Two `#search` instances mounted** (inline desktop + mobile row). → Mitigation: they share one reactive `searchQuery`; behavior is identical client-side filtering. Verify visually that the mobile row is hidden on `md+` and the inline one hidden below `md`, so only one is visible at a time.
- **[Risk] Escape listener leaks**. → Mitigation: add `keydown` listener on mount and remove on unmount; also reset `drawerOpen`/`searchOpen` on route change.
- **[Risk] z-index collisions with the persistent player or dialogs**. → Mitigation: explicit `z-[70]`/`z-[60]`; verify against `PersistentPlayer` and `AppDialog`.
- **[Risk] Icon-only actions lose their text label on mobile**. → Mitigation: the icons (`＋`, `⟳`) are recognizable, and tooltips/aria-labels are kept for accessibility.
- **[Trade-off] Mobile search is a toggle, not always-visible.** → Matches the "expand on demand" pattern; the bar stays minimal.

## Migration Plan

1. `AppHeader.vue`: responsive brand/nav, hamburger + drawer + backdrop, mobile search toggle + row, Escape/route-close handling.
2. `ChannelsView.vue` / `EpisodesView.vue`: wrap action labels in `hidden sm:inline`.
3. Component tests: drawer opens/closes, nav link closes drawer, mobile-only elements hidden on `md`, search toggle expands.
4. `pnpm build`, `pnpm test`, `pnpm lint`; manually verify mobile + desktop layouts.

**Rollback**: revert the frontend changes; no DB/API/config changes.

## Open Questions

None.
