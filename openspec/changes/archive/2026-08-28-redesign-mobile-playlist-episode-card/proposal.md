## Why

Playlist episode cards consume too much vertical space on phones and scatter their metadata and actions across a layout designed for larger screens. The Stitch screen "Rediseño playlist movil" provides a denser, scan-friendly row that should be adopted for the mobile playlist without changing desktop or other episode lists.

## What Changes

- Add a playlist-specific mobile presentation below the existing `sm` breakpoint: reduced drag affordance, compact image-based play/pause, a bold title that scrolls only for the playing episode at a fixed speed in one continuous direction, smaller static channel name, duration, date, progress, and read-only state icons.
- Remove the description, standalone playback controls, and stop control from the mobile playlist row.
- Provide an accessible overflow menu containing exactly Favourite, Remove from playlist, Original link, Reset progress, and Channel view.
- Keep the existing star, rather than the Stitch heart, as the favorite indicator and menu icon; direct row icons remain read-only and their actions live in the menu.
- Place the smaller favorite and playlist status icons below the overflow trigger, always rendering outlined inactive and filled active states, aligned with the duration/date row so titles receive the maximum width.
- Remove the playlist content's mobile horizontal inset and align the publication date to the right edge of the title column.
- Keep the playlist drag handle independently operable while reducing its visible size.
- Preserve the current `sm`-and-wider playlist layout exactly.
- Preserve episode cards in the channel episode list and history view at every viewport size.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `playlist`: define the compact mobile playlist row, its responsive boundary, action access, and coexistence with drag reordering.
- `episode-cards`: define the playlist-only mobile presentation exception while preserving existing card actions, playback state, and all non-playlist presentations.

## Impact

- `frontend/src/components/EpisodeCard.vue`: add an explicit playlist-mobile presentation while retaining the existing default and compact presentations.
- `frontend/src/views/PlaylistView.vue`: select the playlist presentation and coordinate the compact row with the external drag handle.
- `frontend/src/components/EpisodeCard.test.ts` and `frontend/src/views/PlaylistView.test.ts`: cover presentation isolation, mobile action access, metadata/status rendering, and unchanged drag behavior.
- No backend, API, database, player-store, playlist-store, dependency, or desktop UI changes.