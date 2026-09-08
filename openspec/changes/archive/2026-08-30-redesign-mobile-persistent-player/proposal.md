## Why

On narrow viewports the persistent player bar crams a thumbnail, nine controls, a seek bar and two time labels into a single 80px row. The result is unreadable: the episode title is hidden below `sm`, the seek bar collapses to a few pixels, and the controls are too small to hit reliably. Mobile users get the worst version of the most-used surface in the app.

## What Changes

- Introduce a dedicated compact layout for the persistent player bar below the existing `sm` breakpoint (< 640px). The desktop layout (>= 640px) is unchanged.
- The compact layout shows only:
  - a full-width, **read-only** progress bar pinned to the top edge of the bar, rendering played progress and SponsorBlock segments in their existing colors when SponsorBlock is enabled;
  - a square thumbnail on the left;
  - the episode title, horizontally scrolling right-to-left when it overflows;
  - the channel name and the elapsed-time clock (`0:00`, `11:00`, `1:00:00`) on a second line;
  - a play/pause button on the right edge.
- **BREAKING (mobile UI only)**: below `sm`, the previous, next, stop, speed, shuffle, repeat, volume/mute and up-next queue controls are no longer rendered, and the progress bar is no longer tap-to-seek. No mobile affordance replaces them in this change; restoring access to them is deferred to a follow-up proposal.
- The channel name becomes visible in the bar for the first time (sourced from the already-available `channel_title`).
- The title marquee behavior currently local to `EpisodeCard.vue` is extracted into a shared, reusable piece so the bar and the card behave identically (scrolls only while playing, honors `prefers-reduced-motion`, truncates otherwise).

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `persistent-audio-player`: the "Persistent bottom player bar" requirement currently mandates that the bar display stop, scrubber, volume, speed, previous/next and queue controls unconditionally. It must become viewport-dependent: those controls are required only at >= `sm`, while below `sm` a compact composition (read-only progress + thumbnail + scrolling title + channel + elapsed time + play/pause) is required instead. The SponsorBlock marker requirement must also state that markers remain visible on the compact read-only track, and that seeking is unavailable there.

## Impact

- `frontend/src/components/PersistentPlayer.vue` — main rewrite of the template into two responsive compositions; `onSeek` must be inert on the compact track.
- `frontend/src/components/EpisodeCard.vue` — marquee logic and scoped keyframes move to the shared unit.
- New shared marquee component/composable under `frontend/src/components/` (or `frontend/src/lib/`).
- `frontend/src/components/PersistentPlayer.test.ts` — existing selectors (`.fixed.bottom-0`, `button[aria-label="…"]`, `[data-testid="player-sponsorblock-segment"]`) are layout-coupled and must be kept working or updated; new compact-layout tests added.
- `frontend/src/components/EpisodeCard.test.ts` — marquee metric tests follow the extraction.
- No backend, API, store or persistence changes: `usePlayerStore` already exposes `currentLabel`, `progress`, `currentEpisode.channel_title` and the SponsorBlock marker helper.
- i18n: no new user-facing strings expected beyond existing `player.*` keys; the channel name is data, not a label.
