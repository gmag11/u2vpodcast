## Context

The persistent player bar (`frontend/src/components/PersistentPlayer.vue`) renders two viewport-dependent compositions in a single component, switching on the existing 640px breakpoint:

- **Compact** (`sm:hidden`): redesigned — a thin full-width read-only progress track along the top edge, then a single row of thumbnail + scrolling title + chapter + channel·label + play/pause.
- **Wide** (`sm:flex`, `data-testid="player-wide"`): legacy single dense row `h-20` — thumbnail, three-line metadata (title / chapter / `label · duration`), transport controls, an inline centered scrubber, speed, shuffle/repeat, mute/volume, and the queue panel.

The wide composition did not receive the compact redesign. Its scrubber is a short centered element, so precise seeking is awkward, and the episode title is hard-truncated. This change reworks only the wide composition's markup and styles to match the compact visual language, with no behavior or store changes.

## Goals / Non-Goals

**Goals:**
- Full-width interactive scrubber along the wide bar's top edge, thin visually but with an extended hit area for precise seeking.
- Static (non-clickable) thumbnail in the wide bar; no expandable/now-playing view on desktop.
- Two-line metadata: scrolling episode title + `Chapter · Channel`.
- Elapsed/total time readout beside the thumbnail, tabular numerals.
- Keep every existing control in its current horizontal position.

**Non-Goals:**
- No changes to the compact composition or the expanded mobile now-playing view (`PersistentPlayerExpanded.vue`).
- No changes to player store, transport, auto-hide, queue, speed, shuffle/repeat, volume, SponsorBlock, or chapter behavior.
- No backend/API/data-model changes.

## Decisions

### D1: Reuse the compact composition's top-edge strip pattern for the wide scrubber
The compact bar already renders a full-width top strip with chapter and SponsorBlock markers (`PersistentPlayer.vue:236`). The wide scrubber will adopt the same structure: an absolute/full-width thin track (`h-1`) at the very top of the bar, but interactive — clicking/dragging anywhere along it seeks (reusing the existing `onSeek` handler and the wide composition's current marker markup). Rationale: maximum horizontal width for seeking, near-zero vertical cost, and visual consistency with mobile.

Alternative considered: keeping the inline centered scrubber. Rejected — it does not solve the precise-seek problem and leaves the wide bar visually divergent from the compact bar.

### D2: Extended invisible hit area around the thin track
A thin visible strip is a small target. The interactive region will use a taller invisible wrapper (e.g. `py-3`/`-inset-y-*`) so the click/drag target is comfortable while the visual track stays thin. Rationale: matches the "full width, precise, still easy to target" goal.

### D3: Replace the inline centered scrubber and three-line metadata with a two-row layout
The wide bar restructures from one dense row to: (top) the full-width scrubber strip; (below) a single row containing thumbnail + time + two-line metadata on the left and the existing controls (transport, speed, shuffle/repeat, volume, queue) in their current horizontal positions. The removed `label / duration` metadata line moves to a tabular `elapsed / total` readout beside the thumbnail. Rationale: frees the middle of the bar, gives metadata room, and keeps controls where users already expect them.

### D4: Use `ScrollingText` for the wide episode title
The compact bar uses the existing `ScrollingText` component (`ScrollingText.vue`) for the marquee. The wide title will use the same component so overflowing titles scroll while playing and truncate while paused/reduced-motion — identical behavior to the compact bar.

### D5: `Chapter · Channel` secondary line
The second metadata line shows the current chapter title (when within a chapter, via the existing `currentChapterTitle` computed) followed by the channel name (`channel_title`). When no chapter is active, only the channel name shows. Rationale: satisfies the requested vertical order (title, chapter, channel) in two lines.

## Risks / Trade-offs

- [Thin track harder to see] → Keep a contrasting fill and the same accent color as today; the extended hit area preserves usability.
- [Wide bar height growth] → The strip sits on the top border and the content row keeps the current height, so the bar should not grow meaningfully; verify no layout shift.
- [Marquee on desktop could feel distracting] → Same `ScrollingText` reduced-motion/paused truncation already used on mobile; consistent behavior.
- [Chapter·Channel line truncation] → Long channel names truncate (`truncate`), consistent with current behavior.
- [Test churn] → `PersistentPlayer.test.ts` references `player-wide` selectors and metadata; update assertions to the new structure and add wide-specific scenarios.

## Migration Plan

Pure frontend markup/style change within `PersistentPlayer.vue`; no migration needed. Rollback = revert the component change.

## Open Questions

None blocking. (The wide bar keeps all existing controls and the static thumbnail per the user's explicit choices.)
