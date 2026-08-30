## Context

`PersistentPlayer.vue` currently renders two markup branches gated by the `sm:` Tailwind breakpoint (640px): a compact block (`data-testid="player-compact"`) and a wide block (`data-testid="player-wide"`). All state (playing, position, duration, speed, shuffle, repeat, queue) lives in the `usePlayerStore` Pinia store and is already shared by both branches. The wide branch already implements every control the expanded view needs (speed panel, previous/next, interactive scrubber, shuffle toggle, repeat cycle, queue panel) except that its shuffle and repeat are two independent controls, and it also renders volume/mute, which the expanded view must not show. See proposal.md - Why for the motivation.

## Goals / Non-Goals

**Goals:**
- Reuse the existing wide-composition control implementations (speed panel, previous/next, queue panel, interactive scrubber) inside the new expanded view instead of re-implementing them.
- Keep the expanded view a pure presentational overlay: it reads and writes the same `usePlayerStore` state as the compact bar and wide composition; no new store fields for playback itself.
- Keep the compact bar (collapsed) markup and behavior unchanged except for adding the tap-to-expand affordance on the thumbnail.

**Non-Goals:**
- Redesigning the wide (>=640px) composition or its controls.
- Adding favorite/star or overflow-menu functionality (out of scope per proposal.md).
- Persisting whether the expanded view is open across reloads; it always starts closed.
- Building a generic bottom-sheet/modal primitive for reuse elsewhere; a component scoped to this feature is sufficient.

## Decisions

**New sub-component `PersistentPlayerExpanded.vue`, mounted from `PersistentPlayer.vue`, controlled by local `expanded` ref.**
Keeps the already-large `PersistentPlayer.vue` from growing further and isolates the new touch-oriented layout. Alternative considered: inlining the markup in a third `v-if` branch directly in `PersistentPlayer.vue` — rejected, the file is already dense and a separate component makes the mobile-only concern easier to reason about and test in isolation (`PersistentPlayerExpanded.test.ts`).

**The expanded view is a sibling overlay, not a route.**
Toggled by a local boolean (`expanded.value`), rendered with a `Transition` sliding from `translate-y-full` to `translate-y-0`, same technique already used for the bar's own show/hide transition. Alternative considered: a Vue Router route/modal — rejected, this is transient UI state tied to one component tree, not deep-linkable content, and a route would complicate back-button semantics unnecessarily.

**Interactive scrubber and speed panel are extracted only enough to share logic, not markup.**
The existing `onSeek` handler and speed-panel template block operate on generic pixel/ratio math already decoupled from viewport size; the expanded view reuses the same store calls (`player.seek`, `player.setSpeed`) and duplicates the (small) markup with expanded-view-appropriate sizing, rather than forcing one template to serve two very different layouts via conditional classes. Alternative considered: parametrizing the existing wide-composition scrubber/speed-panel markup with props for size — rejected, the compact-bar/wide/expanded layouts differ enough (progress track height, label placement, larger touch targets) that shared markup would need heavy conditional branching for little reuse benefit; sharing the underlying store interactions is what matters for correctness.

**Combined shuffle/repeat control is a small pure-function mapping layer over existing store actions.**
A `mobilePlaybackMode` computed derives one of `'normal' | 'repeat' | 'shuffle'` from `player.shuffle` and `player.repeat`, and a `cycleMobilePlaybackMode()` function calls the existing `player.toggleShuffle()` / `player.cycleRepeat()` (or sets state directly) to land on the next of the three states. No new store state is introduced; the mapping lives in the expanded-view component (or a small shared util) so the store's existing shuffle/repeat model, used by the wide composition and by `playback-modes` scenarios, stays untouched.
Alternative considered: adding a store-level `mobileMode` field — rejected, it would duplicate the source of truth and risk drifting from the real shuffle/repeat state when changed from the wide composition; deriving it on read keeps a single source of truth.

**Open/close driven by tap on thumbnail; close via chevron only (no swipe gesture) for this change.**
Matches the reference image and the user's explicit answer. Swipe-to-dismiss is a plausible future enhancement but adds gesture-conflict complexity (with the horizontal-scroll title, and with page scroll) that isn't required now.

**Auto-close on breakpoint crossing reuses the existing `window.matchMedia('(min-width: 640px)')`-style check.**
`PersistentPlayer.vue` already relies on the `sm:` Tailwind class for branch switching; the expanded view watches the same breakpoint (via a small composable or a `resize`/`matchMedia` listener) and sets `expanded.value = false` when it flips to wide, so the wide composition takes over without a stale expanded overlay.

## Risks / Trade-offs

- [Duplicated markup for scrubber/speed-panel between wide and expanded views increases the surface for visual drift] → Both read/write the same store methods (`player.seek`, `player.setSpeed`), so behavior can't drift even if pixel layout does; covered by scenario-level tests per view.
- [Combining shuffle+repeat into one 3-state mobile control makes repeat-one and shuffle+repeat combinations unreachable from mobile] → Explicitly flagged in proposal.md; the underlying store state is untouched so users can still reach those combinations from a wider viewport (e.g., tablet/desktop, or rotating a mobile device past 640px), and the control shows the closest of the three states rather than erroring.
- [A full-screen overlay component adds another place volume must be deliberately excluded] → Enforced by a scenario test asserting no volume/mute control renders in the expanded view.

## Open Questions

- Exact visual treatment (background blur/dim behind the sheet, corner radius, sheet height) is left to implementation to match the app's existing surface/card styling conventions; it does not affect any specified behavior.
