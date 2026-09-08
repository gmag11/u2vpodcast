## Context

See [proposal.md](proposal.md) for motivation. `PlaylistView.vue` renders every sortable row as an external six-dot drag handle beside `EpisodeCard.vue`, passing `compact` and `queue-source="playlist"`. The card owns no audio element; its script binds playback, progress, playlist membership, favorites, listened state, and notifications to shared stores. The same component also renders channel episodes and history cards.

The Stitch project `U2VPodcast`, screen `Rediseño playlist movil` (`d5821a48ade04f869da5daec2e00a3b1`), is the visual source. Its mobile row uses a small episode image, scrolling title, static channel/duration/date metadata, a bottom progress strip, state icons, a reduced six-dot affordance, and a vertical overflow trigger. U2VPodcast keeps its existing star for favorite state instead of adopting the mockup's heart.

The implementation must preserve the current DOM and visual treatment at `sm` and wider. CSS-responsive branches are preferable to runtime viewport detection because they follow the existing Tailwind breakpoint, react immediately to resizing, and avoid hydration or listener state.

## Goals / Non-Goals

**Goals:**

- Provide a dense, overflow-safe playlist row below `sm` that closely follows the Stitch information hierarchy.
- Reuse the card's existing computed state and handlers so presentation cannot diverge from playback or persistence behavior.
- Reduce the visible drag affordance while keeping drag initiation confined to its accessible interaction area.
- Make the overflow menu keyboard accessible, dismissible, and explicit about toggle state.
- Make breakpoint and view isolation testable without depending on a browser's layout engine.

**Non-Goals:**

- Redesigning the playlist header, navigation, persistent player, channel episode list, or history view.
- Changing playback, reorder, favorite, playlist, progress, or listened-state semantics.
- Changing backend endpoints, stores, data types, translations unrelated to menu labels, or the `sm` breakpoint.
- Reproducing AuraPod branding or unrelated navigation shown in the Stitch mockup.

## Decisions

### 1. Select playlist presentation explicitly at the call site

Add an explicit presentation input to `EpisodeCard` and select the playlist presentation only from `PlaylistView`. Do not derive it from `queueSource`, playlist membership, or viewport width alone. `queueSource` describes playback behavior and must not become a styling switch; an explicit presentation keeps the channel and history call sites unchanged and makes accidental visual spread testable.

Keep the current `compact` contract available for history. The playlist presentation may build on compact sizing internally, but contradictory combinations must be avoided at the playlist call site.

**Alternative considered:** infer the design from `queueSource === 'playlist'`. Rejected because queue provenance is a player concern and could be reused outside the main playlist.

### 2. Add a mobile-only template branch and preserve the desktop branch verbatim

For the selected playlist presentation, render a dedicated row below `sm` and hide the existing card body there. Render the existing card body at `sm` and wider with its current classes, spacing, controls, metadata, and action placement. Non-playlist cards continue to render only the existing body.

The mobile branch follows this order: compact episode image acting as play/pause, a flexible text column with a bold single-line horizontally scrolling title and a smaller normal-weight static channel name, and a narrow trailing column. The trailing column places the vertical overflow trigger above smaller favorite and playlist status icons. Both status icons always render: inactive uses the unfilled icon and muted color, while active uses the filled icon and accent color. Their lower row aligns with the duration/date metadata baseline, and the date is pushed to the right edge of the text column. The existing progress strip remains on the bottom edge. Favorite state uses the current star icon and app tokens, not the mockup's heart. Description, stop, and standalone action buttons are omitted from the mobile playlist branch.

Use stable image, drag-target, action-trigger, and metadata dimensions so long translated text cannot resize controls. The text column must have `min-width: 0`. Apply horizontal scrolling only to the overflowing title of the episode that is actively playing; the current episode must remain static while paused. Keep the channel on its own non-scrolling line and truncate it if required.

Render a second, accessibility-hidden copy of the active title after a gap wider than a normal word space. Move the two-copy track continuously in one direction by exactly the first copy's rendered width plus that gap, so the next copy replaces it without a visible reset. Calculate animation duration from that measured travel distance and a shared pixels-per-second constant rather than assigning a fixed duration, which keeps visual speed independent of title length. Recalculate after title or container-size changes. Respect reduced-motion preferences with a non-animated fallback that still exposes the full title accessibly.

**Alternative considered:** restyle the current single template exclusively with utility classes. Rejected because the mobile information order and action grouping differ enough that conditional class combinations would be fragile and could alter desktop.

### 3. Keep one behavior layer for both presentations

Both template branches call the existing `player`, `playlists`, `favorites`, and progress handlers. Do not create a second store, composable, or duplicate asynchronous action implementation. The mobile image calls the same play/toggle path and retains playlist queue seeding. The menu's Favourite, Remove from playlist, Original link, Reset progress, and Channel view entries invoke the corresponding existing mutation or navigation behavior. Stop remains available in unchanged presentations but is intentionally absent from the mobile playlist branch.

**Alternative considered:** create a standalone `PlaylistEpisodeCard` containing its own behavior. Rejected because it would duplicate synchronization and notification logic already centralized in `EpisodeCard`.

### 4. Group secondary actions in an accessible overflow menu

The mobile playlist branch uses an icon-only vertical-ellipsis trigger with an accessible name, expanded state, and relationship to its menu. It presents exactly five ordered entries: Favourite, Remove from playlist, Original link, Reset progress, and Channel view. The Favourite entry uses the existing star icon. Menu items support keyboard activation, close after an action, close on Escape and outside interaction, and restore focus to the trigger when appropriate. The menu is layered above neighboring rows without changing row height or clipping against the card's overflow treatment.

Playback remains directly available from the episode image. Favorite and playlist state may be communicated by compact icons in the row, but those icons are non-interactive and mutations route through the menu. Reset progress clears saved progress and is distinct from stop; no mobile stop affordance exists.

**Alternative considered:** keep all current icon buttons visible. Rejected because they compete with metadata at phone widths and do not match the Stitch hierarchy.

### 5. Leave the drag handle outside the card

`PlaylistView` remains the owner of sortable rows and the existing handle. On mobile, reduce the visible handle icon and surrounding gap so the image, text, state icons, and overflow trigger fit without page overflow. Preserve an adequately padded hit area, visible focus, and keyboard operation. The menu and all card interactions remain outside the configured drag handle selector, preserving normal scrolling and click behavior.

Remove the playlist main container's horizontal padding below `sm` so the sortable rows can use the drawer width. Preserve the existing inset at `sm` and wider, and retain a mobile inset on the playlist header so its controls do not touch the viewport edge.

**Alternative considered:** move the handle into `EpisodeCard`. Rejected because sorting belongs to the containing list and would couple a reusable card to the draggable library.

### 6. Verify structure, behavior, and rendered responsive output

Component tests assert that only an explicitly selected playlist card includes the mobile branch and overflow menu, while default/compact cards retain their prior structure. Existing behavior tests are reused against mobile menu actions. Playlist tests retain all drag configuration and keyboard checks and add assertions for the selected presentation and independent controls.

Because DOM tests do not apply responsive CSS, browser verification must capture the playlist below and above `sm`, plus channel episodes and history below `sm`. Screenshots and overflow bounds confirm that the mobile row matches Stitch, desktop is unchanged, menus remain visible, and long content does not overlap.

## Risks / Trade-offs

- **Two responsive template branches can drift over time.** -> Keep all behavior in the existing script layer and add tests that exercise the same handlers from the mobile menu.
- **A menu inside an overflow-hidden card can be clipped.** -> Place the menu in a non-clipping positioned wrapper or use the app's established overlay approach while retaining the bottom progress strip.
- **CSS-hidden desktop controls still exist in the DOM.** -> Use `display: none` breakpoint utilities so only the active branch enters the accessibility tree; test active presentation markers and browser accessibility output.
- **The reduced handle may become difficult to acquire.** -> Reduce its visual footprint while preserving an accessible hit area, visible focus, and keyboard operation; validate at 320, 390, and 640 CSS pixels.
- **Moving actions behind a menu adds one interaction.** -> Keep primary play/pause direct and use clear text labels and state indicators for secondary actions.
- **Measured marquee dimensions can change after layout or font loading.** -> Observe the title viewport and text size, recalculate the travel distance, and keep the static first copy as the accessible label and reduced-motion fallback.

## Migration Plan

1. Add the explicit playlist presentation and mobile menu while retaining existing props and behavior.
2. Opt `PlaylistView` into that presentation and adjust mobile-only row spacing.
3. Run focused component and playlist tests, then the frontend typecheck/build.
4. Compare browser screenshots at mobile and desktop breakpoints with the Stitch reference and current desktop baseline.

Deployment requires no data migration. Rollback consists of removing the playlist presentation selection and mobile branch; the unchanged desktop path and stores remain valid throughout.