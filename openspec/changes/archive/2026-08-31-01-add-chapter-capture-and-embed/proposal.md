## Why

yt-dlp already resolves per-video chapters (native YouTube chapters or description timestamps) into the `chapters` field of the JSON it prints during download, but the application discards this field today: `YtVideo` has no `chapters` property, so it is silently dropped by serde, never persisted, and never embedded into the downloaded MP3. When SponsorBlock is enabled and a derived, sponsor-free MP3 is generated for an episode, any chapter markers that a listener's external podcast app might expect would additionally be wrong, because the derived file's timeline is shorter than the original and chapter timestamps are never recalculated for it. This proposal captures chapters at download time and ensures the SponsorBlock-derived MP3 (the file distributed through RSS/downloads) carries chapter markers whose timestamps are correct for that file's own, shorter timeline.

## What Changes

- Add a `chapters` field to the yt-dlp video model (`YtVideo`) and parse `start_time`, `end_time`, and `title` from yt-dlp's existing JSON output; no new yt-dlp invocation or flag is required.
- Persist each episode's raw chapters (as resolved from the original, untrimmed video) at download time, immutably, alongside the episode row.
- Expose raw chapters through the episode read APIs so clients (including the persistent player) can consume them without any SponsorBlock-aware translation.
- When SponsorBlock generates a derived MP3 (`{yt_id}.sponsorblock.{hash}.mp3`), recalculate chapter boundaries against that file's retained-intervals timeline: shift chapters that fall entirely in retained audio, snap boundaries that fall inside a removed interval forward to the next retained point, and drop any chapter that collapses to zero width because it was fully contained in removed audio.
- Embed the recalculated chapters into the derived MP3 as real ID3v2 `CHAP`/`CTOC` frames, written in the same `ffmpeg` invocation that already performs the concat-based trim (no extra pass, no re-encode).
- The original, untrimmed MP3 is never modified; only the SponsorBlock-derived file gains embedded chapters.

## Capabilities

### New Capabilities
- `episode-chapters`: capturing chapters from yt-dlp at download time, persisting them per episode, and exposing raw chapter data through episode APIs.

### Modified Capabilities
- `sponsorblock-integration`: derived MP3 generation additionally recalculates and embeds chapter markers consistent with the retained-intervals timeline used to produce that file.

## Impact

- Backend: `src/models/ytdlp.rs` (`YtVideo` struct, JSON parsing), `src/models/episode.rs` (`Episode` struct, persistence), a new migration adding chapter storage, `src/utils/worker.rs` (episode creation from download), `src/utils/sponsorblock.rs` (`generate_processed_mp3_blocking`, chapter translation function, FFMETADATA1 sidecar generation, ffmpeg invocation), `src/handlers/episodes.rs` (episode API payloads).
- Frontend: `frontend/src/types.ts` (`Episode` type gains a `chapters` field) so downstream player work (tracked separately) can consume it.
- No RSS/feed XML changes in this proposal; ID3-embedded chapters are read directly from the enclosure file by clients that support them. Podcasting 2.0 `<podcast:chapters>` support is tracked as a separate, later change.
