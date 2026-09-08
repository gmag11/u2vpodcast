## Context

See `proposal.md` — Why. Relevant current state (verified in code, not assumed):

- The bar is a single Vue 3 SFC, `frontend/src/components/PersistentPlayer.vue` (~481 lines, `<script setup lang="ts">`, no `<style>` block), mounted app-wide from `App.vue:32`.
- Styling is Tailwind CSS v4 with CSS-first configuration (`frontend/src/app.css`, no `tailwind.config.js`). Semantic tokens (`bg-surface`, `text-text-muted`, `border-outline`, `bg-accent-400`, `bg-sponsorblock`, `bg-sponsorblock-other`) are declared on `:root`/`.dark` and re-exported through `@theme`.
- All state comes from a single Pinia setup store, `usePlayerStore` (`frontend/src/stores/player.ts`). Everything the compact layout needs already exists: `progress` (percent), `currentLabel` (already emits `M:SS` / `H:MM:SS`), `currentEpisode.channel_title`, `currentEpisode.image`, `playing`, `togglePlay`, and `sponsorBlockTimelineMarkers`.
- The bar today is one flex row (`PersistentPlayer.vue:193`, `h-20`) with four `sm:`-prefixed decisions layered on top; there is no separate mobile branch.
- The progress track is click-to-seek only (`onSeek`, `PersistentPlayer.vue:155-160`), on a `role="slider"` wrapper that `player.ts:1214` deliberately excludes from global arrow-key seeking.
- A title marquee already exists but is private to `EpisodeCard.vue` (refs and constants at `:66-70`, activation at `:130-134`, `ResizeObserver` measurement at `:175-197`, markup at `:384-408`, scoped keyframes plus `prefers-reduced-motion` opt-out at `:715-743`).
- `PersistentPlayer.test.ts` selects the bar with `.get('.fixed.bottom-0')` and controls with `button[aria-label="…"]` / `data-testid`. It runs in jsdom, where CSS media queries do not apply and every element in the DOM is queryable regardless of Tailwind breakpoint classes.

Note: the project context string describes the frontend as SvelteKit; the repository is in fact Vue 3 + Vite + Pinia. This design targets the actual stack.

## Goals / Non-Goals

**Goals:**
- One component keeps owning the bar, with two clearly separated template compositions, so the store contract and the mount point are untouched.
- No new state, no new store fields, no backend or API surface.
- Marquee behavior defined once and shared by the card and the bar.
- Existing desktop tests keep passing without behavioral edits.

**Non-Goals:**
- Any mobile affordance for the removed controls (deferred to a follow-up proposal, per the user's explicit decision).
- Reworking the wide composition's visual design.
- Changing the breakpoint system, adding container queries, or introducing a design-token change.
- Touching the `EpisodeCard` read-only progress strip.

## Decisions

### D1 — Breakpoint via CSS classes, both compositions in the DOM

Render both compositions in the template and toggle them with Tailwind's `sm:` variant (`sm:hidden` on the compact block, `hidden sm:flex` on the wide block).

*Why:* it matches the boundary already used by the component and by `EpisodeCard`, requires no resize listener, and gives the spec's "resize across the boundary does not interrupt playback" for free — nothing unmounts the shared `<audio>`, which lives in the store, not the template.

*Alternatives considered:* a `matchMedia` ref driving `v-if`. Rejected: adds a listener and an SSR/initial-paint flash for no benefit, since the audio element is store-owned either way. Splitting into `PersistentPlayerCompact.vue` / `PersistentPlayerWide.vue` was also rejected for now — the two compositions share the SponsorBlock marker computed, the visibility transition and the play/pause handler, and a split would duplicate them or force a props/emit layer.

*Consequence to handle:* in jsdom both compositions are queryable at once, so `getAll('button[aria-label="Play"]')` becomes ambiguous. Mitigated by D4.

### D2 — Compact layout structure

A two-row block inside the existing `fixed bottom-0` root (which must keep those classes — tests depend on them):

1. A full-width track of ~4px pinned to the top edge, outside the horizontal padding, so it truly spans edge to edge as in the reference design.
2. A padded row: square thumbnail (`h-12 w-12 rounded-lg object-cover shrink-0`) → a `min-w-0 flex-1 flex-col` block holding the marquee title and, below it, `channel_title` + `•` + `currentLabel` in `text-text-muted` → play/pause button on the trailing edge with a >= 44px hit target.

`min-w-0` on the flex child is mandatory or the title block will refuse to shrink and push the play button off-screen.

The reference HTML from Stitch uses a different palette (Material-ish `on-surface`, `surface-variant`, Geist at 14px/12px). It is treated as a layout reference only; all colors, radii and fonts come from the existing semantic tokens.

### D3 — Read-only track reuses the marker computation, not the markup

The compact track is a separate element from the wide `role="slider"` scrubber. It:
- has no `@click`, no `role="slider"`, no `tabindex`, and is marked `aria-hidden="true"` (the elapsed clock already conveys position to assistive tech, and an unseekable slider role would be a lie);
- reuses the existing `sponsorBlockMarkers` computed unchanged, so segment geometry and the `bg-sponsorblock` / `bg-sponsorblock-other` colors are identical by construction;
- carries a distinct `data-testid` (e.g. `player-progress-compact`) while its segments keep `data-testid="player-sponsorblock-segment"` so the SponsorBlock colour/geometry assertions still describe the whole bar.

*Why not reuse the same element and disable seeking conditionally:* that would require a JS breakpoint (rejected in D1) and would leave a misleading slider role in the accessibility tree.

### D4 — Test disambiguation strategy

Since both compositions coexist in the DOM under jsdom, existing selectors such as `button[aria-label="Play"]` would match twice. Scope every query by composition using a wrapper `data-testid` (`player-compact` / `player-wide`) and update the existing tests to query within `player-wide`. This is a mechanical change and preserves the assertions themselves.

*Alternative rejected:* giving the compact controls different aria-labels. That would degrade accessibility to serve tests.

### D5 — Marquee extraction

Move the marquee into one shared unit consumed by both `EpisodeCard.vue` and `PersistentPlayer.vue`. Preferred shape: a presentational component (e.g. `ScrollingText.vue`) with props `text: string` and `active: boolean`, encapsulating the viewport/duplicate-copy markup, the `ResizeObserver` measurement, the CSS custom properties (`--*-distance`, `--*-duration`), the keyframes and the `prefers-reduced-motion` guard, and falling back to `truncate` when inactive or non-overflowing.

*Why a component over a composable:* the behavior is inseparable from a specific two-element DOM structure plus scoped keyframes; a composable would still leave both call sites duplicating that markup and CSS.

*Constraint:* `EpisodeCard.test.ts` asserts marquee metrics (`:109-137`, `:194-231`). The extraction must preserve the observable contract — same activation condition (playing AND overflowing), same speed (32 px/s), same gap (32 px) — so those tests keep passing with at most selector updates. Speed and gap constants move with the component; do not re-tune them in this change.

*Risk of scope creep:* if the extraction turns out to disturb `EpisodeCard` more than selector updates, keep `EpisodeCard` untouched and introduce the shared component for the player only, leaving the card migration as a follow-up. The spec does not require the card to change.

## Risks / Trade-offs

- **Mobile users lose access to speed, queue, shuffle, repeat, prev/next, stop and seeking outright.** → Accepted deliberately by the user for this change; the follow-up proposal restoring them should land soon after, and the removal is documented as breaking in the proposal.
- **Both compositions in the DOM double-mount the marquee**, so an off-screen `ResizeObserver` and animation may run for the hidden one. → The hidden composition has zero width under `display: none`, so `scrollWidth === clientWidth === 0` and the marquee stays inactive; verify this in jsdom-free manual testing, and gate activation on a positive measured distance (the existing `titleScrollDistance > 0` check already does).
- **Layout-coupled tests break loudly on any template restructuring.** → Handled by D4; keep `fixed bottom-0` and all aria-labels intact.
- **The reference design implies a shorter bar than the current `h-20`.** → Compact height is a visual detail, not spec'd; pick a height that keeps a >= 44px play target and verify against the screenshot rather than hard-coding the reference's values.
- **`currentLabel` format assumption.** → It already produces `0:00` / `11:09` / `1:00:00`; if a leading-zero or always-hours variant surfaces, fix the store formatter rather than adding a second formatter in the component.

## Migration Plan

Pure frontend, no data or API migration. Work on a dedicated branch (never the default branch). Rollback is reverting the branch. If the app version is bumped as part of the release, `Cargo.toml`, `frontend/package.json` and the `vX.Y.Z` tag in `docker-bake.hcl` must be updated together.
