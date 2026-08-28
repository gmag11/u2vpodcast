## 1. Lock the presentation contract with tests

- [ ] 1.1 Extend `EpisodeCard.test.ts` with playlist-presentation fixtures that verify image-based play/pause, scrolling bold title, smaller static channel, duration, date, read-only state icons, progress treatment, and an overflow trigger while omitting description, standalone controls, and stop; run the focused component test to confirm the assertions discriminate the old markup.
- [ ] 1.2 Add `EpisodeCard.test.ts` cases for opening, keyboard-operating, and dismissing a menu containing exactly Favourite, Remove from playlist, Original link, Reset progress, and Channel view in order; verify the Favourite entry and read-only row state use the current star, and verify accessible names, focus restoration, navigation, notifications, and store/API calls.
- [ ] 1.3 Extend `PlaylistView.test.ts` to verify only playlist cards opt into the new presentation, the visually reduced drag affordance retains an accessible independent hit area, and image/menu/state-icon interaction does not trigger sorting; run the focused playlist test and retain all existing reorder assertions.

## 2. Implement the playlist-specific mobile card

- [ ] 2.1 Add an explicit playlist presentation input to `EpisodeCard.vue` without changing the existing default and `compact` contracts; verify component tests show that channel and history cards do not render playlist-only markup.
- [ ] 2.2 Add the `sm:hidden` dense row from the Stitch hierarchy with stable image/control dimensions, image-based shared-player play/pause, a horizontally scrolling bold title, smaller static channel, duration/date, read-only state icons using the existing favorite star, and no description or stop; verify overflow and reduced-motion tests keep the full title accessible without moving the channel.
- [ ] 2.3 Preserve the current card body as the playlist's `sm`-and-wider branch and as the unchanged branch for all non-playlist cards; verify structural tests assert the desktop classes and existing action placement remain present.
- [ ] 2.4 Implement the mobile overflow menu with exactly Favourite, Remove from playlist, Original link, Reset progress, and Channel view in that order, using the existing star for Favourite and reusing existing mutations/navigation; verify dismissal, focus restoration, action behavior, and absence of a stop item in focused tests.
- [ ] 2.5 Reuse the existing progress, listened, favorite, loading, and current-playback state in the mobile row and keep the read-only bottom progress strip; verify current, partial, listened, favorite, and untouched episode cases in `EpisodeCard.test.ts`.

## 3. Integrate the responsive playlist row

- [ ] 3.1 Opt `PlaylistView.vue` into the explicit playlist presentation and reduce only the below-`sm` handle icon/visual footprint and row gap while preserving its accessible hit area; verify `PlaylistView.test.ts` still passes pointer, touch, keyboard, persistence, rollback, and active-queue reorder cases.
- [ ] 3.2 Add or update only the translations required for overflow trigger/items in both locale files and verify the i18n-backed component tests expose non-empty accessible labels in English and Spanish.
- [ ] 3.3 Run the focused `EpisodeCard.test.ts` and `PlaylistView.test.ts` suites, then the repository's frontend lint, typecheck, and production build; resolve only regressions introduced by this change.
- [ ] 3.4 Start the frontend and use Playwright to capture the playlist at 320, 390, 640, and a desktop width, plus channel episodes and history below `sm`; compare against Stitch screen `d5821a48ade04f869da5daec2e00a3b1`, verify no overlap or horizontal overflow, exercise the menu and playback, and confirm desktop and non-playlist card layouts are unchanged.