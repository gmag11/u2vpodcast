## Why

Downloaded chapters sit in the library but never reach the playlist, so the user has to manually add each one after every sync. Auto-appending freshly downloaded chapters to the end of the playlist turns a channel sync into a ready-to-play queue with zero extra steps.

## What Changes

- When a chapter/episode download completes successfully in the background worker, the episode is automatically appended to the end of the playlist.
- The append reuses the existing playlist "add" semantics: duplicates are rejected (an episode already in the playlist is not added twice), and episodes below the retention/publish floor that get discarded are never appended.
- No UI change: the feature is server-side and always on.

## Capabilities

### New Capabilities
- `auto-playlist-append`: Freshly downloaded episodes are appended to the end of the server-persisted playlist as part of the download flow, using the existing add-episode semantics (append at end, dedupe, respect the publish-date floor).

### Modified Capabilities
<!-- None: the `playlist` spec's add semantics are unchanged; only a new trigger is introduced. -->

## Impact

- `src/utils/worker.rs`: download completion path — after a successful `Episode::new`, call the playlist append for the created episode.
- `src/models/playlist.rs`: reuse `add` (or an internal helper) for the append; no schema changes.
- No migrations required (`playlist_items` table exists).
- No frontend changes.