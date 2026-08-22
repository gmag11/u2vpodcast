## Why

`get_latest` lists a channel with `yt-dlp --dump-json` (full extraction per video). For a large channel — e.g. Defected Music, 4000+ videos — a backfill window of years forces yt-dlp to fully extract **every video in the window** (webpage + JS challenge + PO token per video) before emitting a single line. That turns a backfill into tens of minutes or hours, holds the single YouTube throttle slot for the whole run (blocking every other channel), and produces no visible progress until the entire scan ends.

## What Changes

- `Ytdlp::get_latest` lists the channel with `--flat-playlist`: the API pages come back in seconds with shallow entries (id, title, url, timestamp) instead of fully extracting each video.
- Full episode metadata (description, duration, thumbnail, exact upload date) is obtained **only for the videos that will actually be stored**: inside the date window and not already present.
- The metadata for an in-window video is taken from the very `yt-dlp` run that downloads it (`--print-json`), so no separate extraction pass is needed — one connection per new video, under the existing throttle.
- Rust-side date filtering as a backstop: if `--dateafter` cannot be applied reliably to flat entries, the code filters candidates by the entry timestamp before any per-video work, so out-of-window videos are never detail-fetched.
- Behavior contract is unchanged: same `first`/`last` boundary logic, same `max` retention, same episode rows, same throttle coverage.

## Capabilities

### New Capabilities

- `scalable-channel-listing`: Defines that channel cover listing SHALL be cheap (flat, no per-video extraction) and SHALL defer full episode metadata to the processing step, honoring the date window without extraction.

### Modified Capabilities

(none)

## Impact

- `src/models/ytdlp.rs`: `get_latest` switches to flat listing; `download` gains `--print-json` and returns the parsed metadata alongside the exit status.
- `src/utils/worker.rs`: `process_channel`/`process_episode` filter in-window candidates from the flat listing and build complete episode rows from the download run's metadata.
- `src/models/channel.rs`, episodes table, public API: unchanged.
- Performance: a years-long backfill on a 4000-video channel goes from "hours of invisible scanning, blocking the slot" to "seconds of listing + N runs of (download with embedded metadata)", each throttled and logged per episode.