## Why

Downloaded episodes currently preserve and serve every section of the source audio, including sponsor segments identified by SponsorBlock. The application should retain original MP3 files while offering sponsor-free podcast enclosures and automatic sponsor skipping in the web player, with segment changes reconciled during channel synchronization.

## What Changes

- Fetch `sponsor` segments with `actionType=skip` from the official SponsorBlock API by YouTube video id during channel synchronization.
- Persist the latest successful SponsorBlock snapshot, a canonical content hash, processing state, and derived-media metadata independently from episode records.
- Keep each original MP3 and create a hash-versioned processed MP3 only when SponsorBlock reports sponsor segments; rebuild it only when the canonical segment hash changes.
- Cut and concatenate MP3 frames with FFmpeg stream copy, without re-encoding, and atomically publish only complete derived files.
- Serve the original MP3 when no processed file exists, while retaining the last valid processed file across SponsorBlock or FFmpeg failures.
- Publish the selected original or processed enclosure and its corresponding duration in channel and global RSS feeds.
- Include stored SponsorBlock segments and their hash in episode API payloads, allow an authenticated manual refresh, and make the web player skip those segments while continuing to play the original MP3 timeline.
- Refresh recent-window episodes on each synchronization, including favorites still in that window, while leaving favorites older than the synchronization window unchanged unless manually refreshed.
- Add SponsorBlock data attribution and license information to the README. Frontend attribution is explicitly deferred and remains pending follow-up work.

## Capabilities

### New Capabilities
- `sponsorblock-integration`: SponsorBlock retrieval, snapshot persistence, hash-based reconciliation, derived MP3 lifecycle, failure fallback, refresh policy, and API exposure.

### Modified Capabilities
- `rss-feeds`: Select the processed enclosure when available and publish the actual duration of the selected media representation.
- `global-feed`: Apply the same processed-media selection and duration behavior to the aggregated feed.
- `persistent-audio-player`: Skip stored sponsor intervals on the original MP3 timeline, including playback, seek, and resume transitions.

## Impact

- **Backend:** channel synchronization worker, episode serialization/handlers, feed generation, media cleanup, and a new SponsorBlock client and persistence model.
- **Database:** a new one-to-one SponsorBlock cache table keyed to episodes, with cascade deletion.
- **Media:** original MP3 files remain unchanged; optional hash-versioned processed MP3 files are stored beside them and served through the existing authenticated range-capable media route.
- **Frontend:** episode types/API data and global player behavior; no frontend attribution UI in this change.
- **External systems:** official SponsorBlock API at `https://sponsor.ajay.app`, addressed by `videoID`; FFmpeg/ffprobe from the existing runtime image.
- **Licensing:** SponsorBlock API/database data is CC BY-NC-SA 4.0 and receives README attribution; commercial use requires compatible permission.