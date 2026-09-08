## 1. Branch and baseline

- [x] 1.1 Create a dedicated branch `redesign-mobile-persistent-player` off the default branch and verify `git status` reports the new branch with a clean tree
- [x] 1.2 Run `npm run test` and `npm run lint` in `frontend/` and record the green baseline, so later failures are attributable to this change

## 2. Shared scrolling title (design D5)

- [x] 2.1 Create `frontend/src/components/ScrollingText.vue` with props `text: string` and `active: boolean`, porting the viewport + duplicated `aria-hidden` copy markup, the `scrollWidth`/`clientWidth` + `ResizeObserver` measurement, the `--distance`/`--duration` custom properties, the keyframes and the `prefers-reduced-motion` guard from `EpisodeCard.vue:66-70,130-134,175-197,384-408,715-743`; keep the 32 px gap and 32 px/s speed constants unchanged; verify it truncates when `active` is false or the text fits
- [x] 2.2 Add `frontend/src/components/ScrollingText.test.ts` covering: scrolls when active and overflowing, static when it fits, static when inactive, no animation under `prefers-reduced-motion`; verify the new test file passes
- [x] 2.3 Replace the inline marquee in `EpisodeCard.vue` with `<ScrollingText>`, removing the now-dead refs, constants and scoped keyframes; verify `EpisodeCard.test.ts` still passes (selector-only updates allowed; if the marquee metric assertions at `:109-137,194-231` cannot pass with selector updates alone, revert this task per design D5 and leave the card on its inline implementation)

## 3. Compact composition in the player bar

- [x] 3.1 In `PersistentPlayer.vue`, wrap the current single flex row in a `hidden sm:flex` container tagged `data-testid="player-wide"`, keeping the root `fixed bottom-0 ...` classes, the `<Transition>`, all aria-labels and all existing `data-testid` values untouched; verify `PersistentPlayer.test.ts` still passes after scoping its queries to `player-wide` (task 5.1)
- [x] 3.2 Add the compact composition as a sibling `sm:hidden` container tagged `data-testid="player-compact"`: a full-width ~4px read-only track pinned to the top edge outside horizontal padding, then a padded row with square thumbnail → `min-w-0 flex-1` title/meta column → trailing play/pause button; verify at a 390px viewport that nothing overflows and the play button stays fully visible
- [x] 3.3 Render the compact title with `<ScrollingText :text="title" :active="player.playing" />`; verify a long title scrolls while playing and truncates when paused
- [x] 3.4 Render the compact meta line as `player.currentEpisode.channel_title` + separator + `player.currentLabel`, with no total duration; verify it reads e.g. `VisualPolitik • 11:09` and switches to `1:00:00` past one hour
- [x] 3.5 Render the compact play/pause button reusing the existing `togglePlay` handler and the same aria-label as the wide composition, with a hit target of at least 44×44 px; verify pressing it toggles playback
- [x] 3.6 Render SponsorBlock markers on the compact track from the existing `sponsorBlockMarkers` computed, reusing `bg-sponsorblock` / `bg-sponsorblock-other` and `data-testid="player-sponsorblock-segment"`; verify marker `left`/`width` percentages match the wide scrubber for the same episode
- [x] 3.7 Confirm the compact track has no `@click`, no `role="slider"`, no `tabindex`, and is `aria-hidden="true"`; verify tapping it does not change `player.currentTime`
- [x] 3.8 Confirm no stop, previous, next, speed, shuffle, repeat, volume/mute or queue control exists inside `player-compact`; verify by asserting the compact subtree contains exactly one button

## 4. Styling and tokens

- [x] 4.1 Confirm every color, radius, font size and shadow used in the compact composition comes from existing semantic tokens in `frontend/src/app.css` (no raw hex, no values copied from the Stitch reference); verify by grepping the new markup for `#` and arbitrary-value classes
- [x] 4.2 Verify the compact bar renders correctly in both light and dark themes via the theme toggle

## 5. Tests

- [x] 5.1 Update `PersistentPlayer.test.ts` to scope existing queries within `[data-testid="player-wide"]`; verify all pre-existing assertions still pass unmodified in substance
- [x] 5.2 Add compact-composition tests: reduced control set (task 3.8), elapsed-only clock formatting including the hour rollover, play/pause toggling, SponsorBlock markers present with correct colors and geometry, and no position change on track click; verify the new cases pass
- [x] 5.3 Verify a test asserting that the compact composition renders no `role="slider"` element

## 6. Verification and manual QA

- [x] 6.1 Run `npm run lint` and `npm run test` in `frontend/`; verify both pass with no new warnings
- [x] 6.2 Run `npm run build` in `frontend/`; verify the production build succeeds
- [x] 6.3 Manually verify on a 390px-wide viewport with a long-titled episode: full-width progress bar, readable title scrolling, channel + clock legible, play/pause reachable with one thumb, nothing clipped — compare against the reference screenshot in the proposal thread
- [x] 6.4 Manually resize the browser across the 640px boundary during playback; verify the composition swaps and audio continues without interruption or position reset
- [x] 6.5 Manually verify with SponsorBlock enabled that segments appear on the compact track in the expected colors, and with it disabled that no markers appear
- [x] 6.6 Run `openspec validate redesign-mobile-persistent-player --strict`; verify it reports the change as valid
