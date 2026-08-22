## Why

The current sync strategy is date-boundary based: it lists videos published since the newest stored episode (or `channel.first`) and downloads everything new. That breaks when episodes are pruned or gaps appear (raising the retention limit does not recover older episodes), and it makes the listing cost grow with the window. For large channels the listing itself is also needlessly heavy (full per-video extraction).

A channel with a 3-year `first` and `max=20` should simply make sure the **20 most recent videos** are stored, regardless of what was downloaded before. If those 20 already exist, nothing happens; if some are missing, only those are downloaded. Raising `max` to 30 automatically pulls the 10 older ones.

## What Changes

- **Count-window sync instead of date-window sync:** the candidate window is the `max` most recent videos (newest-first), not "everything since the last download". No dependence on the last-downloaded date.
- **Selection rule:** the candidate window is the first `max` entries of the flat listing (the `/videos` tab is newest-first, so "the most recent `max`"); already-stored ones are skipped; the missing ones are downloaded (audio + full metadata in a single throttled run). Dates are used only for exclusions and the `first` floor.
- **Date sanity:** upcoming (`is_upcoming`), live (`is_live`), and future-dated entries are excluded; entries without a parseable date are kept but ranked last (their real date is resolved at download and re-checked against the `first` floor).
- **`first` is now a floor** (never scan/download older videos), not the "last downloaded" boundary.
- **Bounded flat listing:** `--flat-playlist` limited to `max + margin` pages/entries — cheap per cycle, independent of channel age.
- Retention `max` behaviour unchanged for pruning when lowered; raising `max` backfills the missing older episodes.

## Capabilities

### New Capabilities

- `scalable-channel-listing`: Defines the bounded, count-window channel sync: flat listing capped to the target count, top-N selection by date, upcoming/future exclusion, and per-video metadata deferred to the download run.

### Modified Capabilities

(none)

## Impact

- `src/models/ytdlp.rs`: `get_latest` becomes `list_videos_wanted(url, count)` (flat, `--playlist-items` capped, no `--dateafter`); `download` keeps `--print-json` and returns `(ExitStatus, YtVideo)`.
- `src/utils/worker.rs`: candidate selection (date ordering, upcoming/future exclusion, `first` floor), per-video processing (skip stored, download missing, floor re-check).
- `src/models/ytdlp.rs` `YtVideo`: tolerates missing fields; gains optional `release_date`/`live_status`.
- Semantics: raising `max` recovers older missing episodes; `first` acts as a hard floor.
- Per cycle cost: constant (flat listing of `max + margin` entries), not proportional to channel age or window.