## Context

The expanded "now playing" view (`PersistentPlayerExpanded.vue`) already displays thumbnail, title, scrubber (extended with chapter tick marks by `02-add-player-chapter-markers`), speed control, mode toggle, queue toggle, and transport controls. Episodes gain a `chapters: Array<{start, end, title}>` field from `01-add-chapter-capture-and-embed`.

## Goals / Non-Goals

**Goals:**
- Add a scannable, tappable list of chapters to the expanded view only (not the compact bar or wide composition, which have no room for a list).
- Keep the "current chapter" lookup as a small reusable helper since `04-add-player-current-chapter-label` may need the same computation.

**Non-Goals:**
- Any change to the compact bar or wide composition layout.
- The now-playing chapter label near the episode title (tracked in `04-add-player-current-chapter-label`).
- Prev/next-chapter transport buttons (tracked in `05-add-player-chapter-navigation`).

## Decisions

**Add a `currentChapterIndex(currentTime, chapters)` pure function in `frontend/src/stores/player.ts`** (or reuse it if `04-add-player-current-chapter-label` already introduced an equivalent helper) returning the index of the chapter whose `[start, end)` contains `currentTime`, or `-1` if none. Both this change and `04` can share it; whichever lands first should add it, and the other should reuse rather than duplicate.

**Render the Chapters section below the transport controls**, as a simple scrollable list within the existing expanded view's layout, each row showing title and a formatted start time (reusing the existing time-formatting helpers already used for elapsed/remaining labels).

**Seeking from a chapter row calls the same `player.seek()` used elsewhere**, inheriting existing rejected-interval skip behavior with no special-casing.

## Risks / Trade-offs

- [Risk] Episodes with many chapters could make the expanded view tall / require internal scrolling → Mitigation: constrain the Chapters section to a fixed max height with internal scroll, consistent with how the existing queue panel handles a long "Up next" list.
