## 1. Header responsive layout

- [ ] 1.1 In `frontend/src/components/AppHeader.vue`, hide the "U2V Podcast" wordmark below `md` (`hidden md:inline`) while keeping the brand icon.
- [ ] 1.2 Hide the inline nav links (Channels, History) below `md` (`hidden md:flex`) and add a `md:hidden` hamburger button in the bar.
- [ ] 1.3 Keep the theme toggle visible at all sizes; verify it still sits correctly in the mobile bar.

## 2. Navigation drawer

- [ ] 2.1 Add `drawerOpen` state to `AppHeader.vue` and a backdrop + right-side fixed drawer (top-0 right-0 h-full w-72) rendered only when open, with `z-[60]` backdrop and `z-[70]` drawer.
- [ ] 2.2 Render in the drawer: the logged-in user's avatar and name (from `useAuthStore`), `RouterLink` items for Channels and History with the same active styling as the inline links, and a Logout button reusing the existing logout handler.
- [ ] 2.3 Close the drawer on: backdrop click, Escape (`keydown` listener added on mount and removed on unmount), `RouterLink` selection, and a `watch` on `route`.
- [ ] 2.4 Verify `PersistentPlayer`/`AppDialog` z-indexes so the backdrop and drawer layer above them.

## 3. Icon-only per-view actions

- [ ] 3.1 In `frontend/src/views/ChannelsView.vue`, wrap the "Create New" label text in `<span class="hidden sm:inline">` so the button is icon-only on mobile and text+icon on `sm+`.
- [ ] 3.2 In `frontend/src/views/EpisodesView.vue`, do the same for the "Refresh" label text.

## 4. Mobile search

- [ ] 4.1 In `AppHeader.vue`, when `$slots.search` is present, render a `md:hidden` magnifier toggle in the bar and an expandable, full-width, absolute search row below the header (`md:hidden`) shown when `searchOpen`; keep the existing inline `md:flex` search for desktop.
- [ ] 4.2 Toggle `searchOpen` on the magnifier button and close the row on route change.
- [ ] 4.3 Confirm typing in the mobile search row filters the view's list exactly like the desktop input (shared `v-model` state).

## 5. Tests and verification

- [ ] 5.1 Add component tests (e.g. `frontend/src/components/AppHeader.test.ts`) covering: drawer opens and closes, selecting a nav link closes the drawer, mobile-only elements are present/absent by breakpoint class, and the search toggle expands the mobile search row.
- [ ] 5.2 Run `pnpm build`, `pnpm test`, and `pnpm lint`; confirm no type, test, or style errors.
- [ ] 5.3 Manually verify mobile (<`md`): icon-only brand and actions, no inline nav, hamburger opens the drawer with user/nav/Logout, backdrop and Escape close it, theme toggle works, search toggle expands/collapses the Channels search.
- [ ] 5.4 Manually verify desktop (`md+`): header identical to before, inline nav, inline search, text+icon actions, theme and logout unchanged.
