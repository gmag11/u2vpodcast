## Context

The persistent player (Vue 3 SPA, Tailwind styling, Pinia `player` store) exposes the progress track in three compositions:

- **Compact** (`PersistentPlayer.vue`, `sm:hidden`, mobile): a thin read-only track. No seek interaction.
- **Wide** (`PersistentPlayer.vue`, `hidden sm:block`): an interactive scrubber (`role="slider"`, `@click="onSeek"`) with a thin visible track and a larger invisible hit area.
- **Expanded** (`PersistentPlayerExpanded.vue`): an interactive scrubber identical in behavior to the wide one.

All three already render the same overlay markers: SponsorBlock segment bars and chapter markers with tooltips. Seeking is performed via `player.seek(seconds)` in `frontend/src/stores/player.ts:1255`, which applies the existing rejected-interval skip (`sponsorBlockSkipTarget`) and persists progress via the existing throttled saves. Time labels use a `currentLabel`/readout formatter (`elapsed / total`).

None of the compositions shows a visible, draggable thumb; wide/expanded only support click-to-seek, and compact is entirely read-only.

## Goals / Non-Goals

**Goals:**
- A visible draggable thumb on the progress track of the wide (desktop) and expanded (mobile expanded) compositions.
- Drag-preview: while dragging, thumb follows pointer and a tooltip shows the target time; no seek until release.
- On release, seek to the previewed position (reusing `player.seek`, so SponsorBlock skip and progress persistence apply).
- Keep existing click-to-seek working.
- Reuse the existing time-label format and marker rendering.
- Leave the compact (mobile collapsed) track read-only, exactly as today.

**Non-Goals:**
- No backend, store-state, or API changes.
- No changes to chapter/SponsorBlock marker logic or their tooltips.
- No scrubbing on the compact (mobile collapsed) composition.
- No keyboard/slider `role` semantic overhaul beyond what's needed for the thumb (a11y niceties like arrow-key scrubbing are out of scope).

## Decisions

### 1. Shared `ProgressScrubber.vue` component for the wide and expanded tracks
Extract the repeated progress-track markup (track, fill, SponsorBlock markers, chapter markers, thumb, tooltip) into one reusable component. Props: `progress` (0–100), `duration`, `sponsorBlockMarkers`, `chapterMarkers`, `thin` (wide track uses the h-1 track, expanded h-1.5), `dataTestId`, `ariaLabel`. Emits `seek(seconds)`. The wide and expanded compositions render it, keeping markup and drag logic in one place. The compact track keeps its existing inline read-only markup.
- **Why**: The wide and expanded tracks are near-identical already; a shared component removes duplication of the drag/preview logic and keeps behavior consistent.
- **Alternative**: Add thumb + logic inline to each of the two templates. Rejected: duplicates logic and risks drift.

### 2. Drag handled with Pointer Events (pointerdown/move/up) on the component root
Use the Pointer Events API with `setPointerCapture` on the track element so the drag continues even when the pointer leaves the element. On `pointerdown` start a drag and record the initial clientX; on `pointermove` compute `ratio = clamp((clientX - rect.left) / rect.width, 0, 1)` and store it in a local reactive `dragRatio`; on `pointerup` (or `pointercancel`) commit `emit('seek', dragRatio * duration)` and clear the preview state. A click that does not move the pointer meaningfully falls through to a plain seek (click-to-seek preserved).
- **Why**: Pointer Events unify mouse and touch, and `setPointerCapture` is the standard way to keep a drag alive outside the element bounds.
- **Alternative**: `mousedown`/`touchstart` pairs. Rejected: more branches, no pointer capture benefits.

### 3. Drag preview is local component state; live position only updates on seek
During a drag, the thumb and tooltip are driven by the local `dragRatio` (the preview), not by `player.progress`. Playback continues (or is unaffected) while dragging; the actual seek fires only on release. This matches the spec ("time of the point that will be jumped to if released") and avoids seeking on every pixel of movement.
- **Why**: Matches requirement and avoids spamming `audio.currentTime` during the drag.
- **Trade-off**: The thumb may briefly jump from the "live" playback position to the release target; acceptable and standard for scrubber UX.

### 4. Tooltip renders the `elapsed / total` label for the preview position
Reuse the existing time formatter used for `currentLabel`. Compute the preview seconds as `dragRatio * duration` and render `formatTime(previewSeconds) / formatTime(duration)` above the thumb. Clamp tooltip positioning at track ends (reuse the left/right anchor pattern already used for chapter-marker tooltips in `PersistentPlayerExpanded.vue`).
- **Why**: Consistency with the existing readout; minimal new strings (no i18n additions needed beyond what exists).
- **Alternative**: Show only elapsed seconds. Rejected: inconsistent with the player's `elapsed / total` readout.

### 5. Compact track stays read-only
The compact (mobile collapsed) composition is intentionally left untouched: no thumb, no drag, no seek. The user only wants scrubbing on desktop (wide) and the mobile expanded view. Its read-only markers and `aria-hidden` track remain as-is.
- **Why**: Explicit product scope; the compact bar is a minimal collapsed affordance and scrubbing there was not requested.

### 6. Guard on unknown duration
The component disables thumb dragging and ignores seek when `duration <= 0` or not finite, matching existing `onSeek` guards (`if (player.duration <= 0) return`).

## Risks / Trade-offs

- **Drag vs. click ambiguity** → Use a small movement threshold before treating a press as a drag; a press-and-release without meaningful movement is treated as a click-to-seek.
- **Seek jumps to SponsorBlock-skipped target** → The previewed time reflects the raw pointer position, but release seeks through `player.seek` which may skip to a later time. Acceptable; tooltip indicates the pointer time, and the rejected-interval behavior is existing/expected. Mitigation: document that the tooltip is the pointer target, not necessarily the final landed time.
- **Touch + scroll conflict on mobile expanded track** → The scrubber sets `touch-action: none` so horizontal dragging does not scroll the view, and `setPointerCapture` keeps the drag responsive outside the element.
- **Accessibility** → The wide/expanded tracks keep `role="slider"`; keyboard-arrow scrubbing is intentionally out of scope (non-goal).

## Migration Plan

Pure frontend addition; no data migration. Deploy with the SPA build. Rollback: revert the component changes; wide/expanded return to click-only and compact stays read-only (prior behavior). No backend version bump required.
