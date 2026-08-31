## Why

Once episodes carry chapter data (tracked in `01-add-chapter-capture-and-embed`), listeners using the web player have no way to see where a chapter begins or jump to it — the seek bar only shows playback progress and SponsorBlock segment ranges today. Since the web `<audio>` element always streams the original, untrimmed file and only skips live over rejected SponsorBlock ranges (it never streams the SponsorBlock-derived file), chapter markers can be plotted directly from the episode's stored, untranslated chapter times with no coordinate translation.

## What Changes

- Render a vertical tick mark on the persistent player's seek bar (wide composition) and the expanded "now playing" view's scrubber at each chapter's original start time, positioned the same way the existing SponsorBlock range markers are (percentage of the original media duration), but as a point marker rather than a range.
- Render the same chapter tick marks on the compact composition's read-only progress track, consistent with how it already shows SponsorBlock markers without accepting seek interaction.
- Clicking or tapping a chapter marker on an interactive scrubber (wide or expanded) seeks playback to that chapter's start time, subject to the existing SponsorBlock rejected-interval skip behavior if that time falls inside a rejected segment.
- Hovering or keyboard-focusing an interactive chapter marker shows its chapter title in an immediate, styled tooltip rather than relying on the browser's delayed native `title` popup.
- An episode with no stored chapters renders no chapter markers; no behavior changes for such episodes.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `persistent-audio-player`: adds chapter marker rendering and seek-to-chapter interaction to the existing seek bar/scrubber requirements, as a new requirement within this capability.

## Impact

- Frontend only: `frontend/src/stores/player.ts` (a new `chapterTimelineMarkers()` helper, mirroring `sponsorBlockTimelineMarkers()`), `frontend/src/components/PersistentPlayer.vue` (wide and compact tracks), `frontend/src/components/PersistentPlayerExpanded.vue` (expanded scrubber), `frontend/src/types.ts` (consumes the `chapters` field added by `01-add-chapter-capture-and-embed`).
- No backend changes; depends on `01-add-chapter-capture-and-embed` having landed the `chapters` field on the episode API response, but does not depend on its SponsorBlock-embedding work.
