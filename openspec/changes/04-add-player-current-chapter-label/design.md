## Context

The wide composition and expanded view both show the episode title prominently. Episodes gain a `chapters` field from `01-add-chapter-capture-and-embed`. `03-add-player-chapter-list` may introduce a `currentChapterIndex`/`currentChapter` helper in `frontend/src/stores/player.ts` for the same purpose (identifying which chapter contains `currentTime`).

## Goals / Non-Goals

**Goals:**
- Reuse the current-chapter lookup helper rather than duplicating it, regardless of implementation order relative to `03-add-player-chapter-list`.
- Keep the label subtle (secondary text, not competing visually with the episode title).

**Non-Goals:**
- The compact composition, which is space-constrained and already has a closed, spec'd list of elements (see proposal.md).
- The chapter list itself (`03-add-player-chapter-list`) and prev/next-chapter controls (`05-add-player-chapter-navigation`).

## Decisions

**Reuse `currentChapterIndex`/`currentChapter` from `03-add-player-chapter-list` if present; otherwise introduce it here.** Both changes need the identical computation (which chapter's `[start, end)` contains `currentTime`); whichever change is implemented first should add it to `frontend/src/stores/player.ts`, and the other should import and reuse it instead of re-deriving the logic.

**Render the label as a computed string derived from `currentChapterIndex`**, falling back to an empty/absent state (no label, no reserved space) when the index is `-1` (before the first chapter or no chapters at all) — mirrors how other optional player UI (e.g., resume label on `EpisodeCard`) conditionally renders nothing rather than an empty placeholder.

## Risks / Trade-offs

- [Risk] A very long chapter title could crowd the wide bar's already-tight horizontal space next to title/controls → Mitigation: truncate the chapter label with ellipsis (no scrolling animation, unlike the episode title), since it's secondary information.
