## Why

A previous change reduced the compact (mobile, <640px) persistent player bar to a minimal set of information (thumbnail, title, channel, elapsed time, play/pause) to keep the bar unobtrusive. That reduction removed access to controls (speed, previous/next, seek, shuffle/repeat, queue) that are still available on wider viewports. Mobile users currently have no way to reach these controls at all. Tapping the thumbnail should open a full-screen "now playing" view that restores this functionality in a layout designed for touch, without re-adding it to the persistent bar itself.

## What Changes

- Add a full-screen expanded "now playing" view for the compact (<640px) composition, opened by tapping the persistent bar's thumbnail.
- The expanded view slides up from the bottom over the current page content and shows: a close affordance (chevron-down, top-left), episode thumbnail (large), title, channel name and date, playback speed control, shuffle/repeat control, "Up next" queue toggle, an interactive progress bar with elapsed/remaining time labels, and transport controls (previous, seek back 10s, play/pause, seek forward 10s, next).
- Unlike the compact bar's read-only progress track, the expanded view's progress bar SHALL be a fully interactive scrubber (tap/drag to seek to any position), matching the wide-composition scrubber's capability.
- Volume/mute controls are explicitly out of scope for the expanded view (not relevant on mobile devices, which have hardware volume control).
- The expanded view combines shuffle and repeat into a single toggle control cycling through three mutually exclusive states: normal order, repeat, and shuffle (see Impact for the simplification this implies versus the existing independent shuffle/repeat-all/repeat-one model).
- Favorite/star toggle and any overflow/menu action are explicitly out of scope for this change (no such capability exists elsewhere in the app today).
- The expanded view is only reachable from, and only replaces, the compact composition; the wide composition (>=640px) is unchanged.
- Closing the expanded view (chevron button) returns to the compact bar without interrupting playback.

## Capabilities

### New Capabilities
(none — this extends the existing persistent player bar capability rather than introducing a standalone one)

### Modified Capabilities
- `persistent-audio-player`: adds a mobile-only expanded "now playing" view reachable by tapping the compact bar's thumbnail, specifying its layout, contained controls, interactive scrubber, and open/close behavior.
- `playback-modes`: adds a mobile expanded-view-only combined shuffle/repeat control that cycles through normal/repeat/shuffle as a single affordance, alongside the existing independent shuffle toggle and none/all/one repeat cycle used elsewhere.

## Impact

- Frontend Vue 3 SPA: the persistent player bar component (compact composition) and its child components; the existing wide-composition controls (speed, previous/next, up-next queue toggle) are reused/relocated into the new expanded view rather than reimplemented.
- Global audio player Pinia store: no new state needed beyond what the wide composition already reads (position, duration, speed, shuffle, repeat, queue); the combined shuffle/repeat toggle needs a small amount of view-level logic to map the store's two independent flags (shuffle boolean, repeat none/all/one) onto three simplified UI states and back.
- **Design/scope note to flag for review**: mapping the store's independent shuffle (on/off) and repeat (none/all/one) flags onto a single 3-state mobile toggle is a real reduction in expressiveness — it makes `repeat-one` and any combination of shuffle+repeat unreachable from the mobile expanded view (they remain reachable, and preserved, from the wide composition). This is called out explicitly since it was a deliberate ask; worth reconsidering if repeat-one-on-mobile matters to users before implementation starts.
