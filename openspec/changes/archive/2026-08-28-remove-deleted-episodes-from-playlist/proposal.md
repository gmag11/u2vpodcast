## Why

Deleting an episode today (via the channel retention-limit prune worker, or deleting a whole channel) leaves its row in `playlist_items` behind. The playlist read hides these orphaned rows (`INNER JOIN` to `episodes`), so they are invisible in the API response, but they still occupy a stored `position` and are never reindexed until the next unrelated `remove`/`reorder` call — silently corrupting position bookkeeping and (for `reorder`) requiring the submitted set to match the live episodes exactly, which is confusing to debug. The playlist should stay consistent with the episode library at all times: an episode that no longer exists must not remain a phantom playlist entry.

Note: the companion behavior the user asked to confirm — that finishing playback of a playlist episode removes it from the playlist — is already implemented and tested (`playlist` capability, "Finishing an episode removes it from the playlist and marks it listened"; see `frontend/src/stores/player.ts` and `frontend/src/stores/player.test.ts`). No change is needed there; this proposal only closes the episode-deletion gap.

## What Changes

- When an episode row is deleted for any reason (channel retention-limit prune, and bulk deletion of all of a channel's episodes when the channel itself is deleted), the corresponding `playlist_items` row (if any) is deleted in the same transaction and the remaining playlist positions are reindexed contiguously.
- No API contract change: the playlist read/add/remove/reorder endpoints behave as documented today; this only guarantees the backing table never accumulates orphaned rows.

## Capabilities

### New Capabilities
(none)

### Modified Capabilities
- `playlist`: add a requirement that deleting an episode (individually or as part of a channel deletion) also removes it from the playlist, reindexing remaining positions.

## Impact

- `src/models/episode.rs` (`Episode::remove`, used by the retention-limit worker) — must also delete the playlist row and reindex.
- `src/models/channel.rs` (`Channel::delete`, bulk `DELETE FROM episodes WHERE channel_id = $1`) — must also delete any playlist rows for those episodes and reindex.
- `src/models/playlist.rs` — reuse/expose the existing `reindex` helper for these new call sites.
- No frontend or API surface changes; no migration needed (no schema change).
