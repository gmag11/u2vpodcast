## Why

With chapters visible as timeline markers, a list, and a current-chapter label, the natural next step is letting listeners jump directly between chapters without dragging the scrubber — mirroring the existing prev/next-*episode* transport controls, but scoped to chapters within the current episode.

## What Changes

- Add previous-chapter and next-chapter controls to the expanded "now playing" view, distinct from the existing previous/next-*episode* controls, visible only when the current episode has stored chapters.
- Activating next-chapter seeks to the start of the chapter after the one containing the current playback position; if already in the last chapter, it seeks to the end of the episode's chapters (or is disabled, per design).
- Activating previous-chapter restarts the current chapter (seeks to its start) when more than 3 seconds have elapsed since that chapter's start; otherwise it seeks to the previous chapter's start — mirroring the existing 3-second restart-vs-previous threshold already used by the episode-level previous control.
- Both controls respect existing SponsorBlock rejected-interval skip behavior after seeking.
- An episode with no stored chapters shows neither control.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `persistent-audio-player`: adds prev/next-chapter transport controls to the expanded now-playing view.

## Impact

- Frontend only: `frontend/src/components/PersistentPlayerExpanded.vue`, `frontend/src/stores/player.ts` (reuses `currentChapterIndex` from `03-add-player-chapter-list`/`04-add-player-current-chapter-label`, adds seek-to-chapter-offset logic).
- Depends on `01-add-chapter-capture-and-embed` for the `chapters` field.
