## Context

The expanded view already has episode-level previous/next controls (`auto-advance`, `up-next-queue` capabilities) and, per `system-media-controls`, an established 3-second restart-vs-previous threshold for episode-level previous. Chapters within an episode gain the same shape of navigation, scoped to `currentEpisode.chapters` instead of the queue.

## Goals / Non-Goals

**Goals:**
- Reuse `currentChapterIndex` (from `03-add-player-chapter-list`/`04-add-player-current-chapter-label`) to locate the current chapter, and apply the same 3-second threshold convention already established for episode-level previous, for consistency of mental model.

**Non-Goals:**
- Any change to the existing episode-level previous/next controls or their behavior.
- System media session (hardware/OS) integration for chapter navigation — out of scope; `system-media-controls` maps hardware previous/next to episode-level navigation only, unchanged by this proposal.

## Decisions

**Add `nextChapterStart(currentTime, chapters)` and `previousChapterSeekTarget(currentTime, chapters)` pure functions in `frontend/src/stores/player.ts`.** Both operate purely on `currentChapterIndex` plus the 3-second threshold, returning `null` when the corresponding control should be disabled (last chapter for next; first chapter within 3s of its start for previous). Pure functions keep the threshold logic unit-testable without mounting the component.

**Chapter transport buttons are placed in the expanded view only**, near the existing episode-level previous/next controls but visually grouped with the Chapters section (`03-add-player-chapter-list`) to avoid confusing users about which "previous/next" a button controls — episode-level vs chapter-level.

## Risks / Trade-offs

- [Risk] Visual confusion between episode-level and chapter-level previous/next controls sitting near each other → Mitigation: distinct icon treatment (e.g., a chapter-specific icon or label) and grouping near the Chapters section rather than the primary transport row.
