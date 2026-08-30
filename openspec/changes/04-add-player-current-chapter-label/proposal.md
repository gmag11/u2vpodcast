## Why

Once episodes carry chapter data, listeners have no lightweight, always-visible indication of which chapter is currently playing without opening a chapter list or reading timeline tick marks. A short "now playing: <chapter title>" label near the episode title gives this at a glance, matching the pattern used by mainstream podcast apps.

## What Changes

- Display the current chapter's title as a secondary label near the episode title in the persistent player's wide composition and in the expanded "now playing" view, updating live as playback crosses chapter boundaries.
- The label is present only when the current episode has stored chapters; absent otherwise, with no layout shift for episodes without chapters.
- The compact composition (viewport < 640px) is explicitly out of scope: it already renders a closed, space-constrained set of elements (`persistent-audio-player`'s compact-composition requirement lists them exhaustively), and this change does not add to that list.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `persistent-audio-player`: adds a current-chapter label requirement.

## Impact

- Frontend only: `frontend/src/components/PersistentPlayer.vue`, `frontend/src/components/PersistentPlayerExpanded.vue`, `frontend/src/stores/player.ts` (reuses or introduces the `currentChapterIndex`/`currentChapter` helper also used by `03-add-player-chapter-list`).
- Depends on `01-add-chapter-capture-and-embed` for the `chapters` field on episodes.
