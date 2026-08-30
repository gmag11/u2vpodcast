## Context

yt-dlp's JSON output (already parsed via `parse_dump_output` in `src/models/ytdlp.rs`, driven by `--print-json`) includes a `chapters` array whenever the source video has chapters, but `YtVideo` has no field for it, so serde silently discards it (no `deny_unknown_fields`). No chapter data is stored anywhere today: the `episodes` table has no chapter column, and the SponsorBlock-derived MP3 (`generate_processed_mp3_blocking` in `src/utils/sponsorblock.rs`) is produced via an `ffmpeg -f concat ... -c:a copy` pass with no chapter metadata. SponsorBlock trimming works by inverting rejected segments into "retained intervals" (`retained_intervals`) and stitching them back together via an `ffconcat` manifest; the resulting file's timeline is shorter than the original whenever any interval was removed.

Separately, the web persistent player never streams the SponsorBlock-derived file — `mediaUrl()` in `frontend/src/stores/player.ts` always requests `/media/{channel_slug}/{yt_id}.mp3` (the original), and playback instead skips live over rejected ranges. Only the RSS feed's enclosure (`selected_media()` in `src/models/episode.rs`, used by `episode_item()` in `src/handlers/feed.rs`) picks the derived file. This means chapter translation is only ever needed for the physically-cut file that RSS/downloads serve to external clients; the web player (tracked in change `02-add-player-chapter-markers`) consumes untranslated, original-timeline chapters directly.

## Goals / Non-Goals

**Goals:**
- Capture chapters yt-dlp already resolves, with no additional yt-dlp invocation.
- Persist raw chapters once, immutably, as part of an episode's own data (not the SponsorBlock cache, which is re-fetched and invalidated on a different lifecycle).
- Correctly recalculate chapter boundaries for the SponsorBlock-derived MP3's shorter timeline, and embed them as real MP3 chapter metadata so external podcast apps display correct chapters for the file they actually receive.

**Non-Goals:**
- Changing what the web player streams or how it displays chapters (separate change).
- Adding Podcasting 2.0 `<podcast:chapters>` RSS support (separate change).
- Re-encoding audio, or any change to the SponsorBlock category/rejection logic itself.

## Decisions

**Store raw chapters as an immutable column on `episodes`, not in `sponsorblock_cache`.** Chapters are a property of the source video, resolved once at download time, independent of whether SponsorBlock is enabled, refreshed, or reconfigured. This mirrors how `duration`, `webpage_url`, and `title` are plain columns set at creation, not part of the SponsorBlock side-table (`sponsorblock_cache`) whose whole purpose is to track externally-refreshable, re-fetchable state. Chapters are stored as a JSON array (`chapters_json TEXT NULL`), following the existing precedent of storing segment lists as JSON text (`sponsorblock_cache.segments_json`) rather than a normalized child table, since chapters are always read/written as a whole ordered list per episode and never queried individually.

**Translate chapters with a monotonic step function over `retained_intervals`, computed at derived-MP3-generation time, not cached separately.** `retained_intervals(sponsor_segments, original_duration)` already produces the ordered list of kept `[start, end)` ranges used to build the `ffconcat` manifest. Chapter translation reuses this exact list (no new interval computation) via:

```
translate(t):
  cumulative = 0
  for interval in retained_intervals:
    if t <= interval.start: return cumulative
    if t <= interval.end:   return cumulative + (t - interval.start)
    cumulative += interval.end - interval.start
  return cumulative
```

This is monotonic non-decreasing and snaps any instant that falls inside removed audio forward to the point where that removal lands in the output. Applying it to both a chapter's `start` and `end` yields the translated chapter; if `translate(start) == translate(end)`, the chapter is fully contained in removed audio and is dropped. This runs only when a derived MP3 is actually (re)generated (i.e., the processing hash changed), which is exactly when translated chapters would need to change anyway — there is no independent caching or invalidation to get wrong.

**Embed via an FFMETADATA1 sidecar muxed in the same `ffmpeg` invocation that performs the concat trim.** Verified working: `ffmpeg -f concat -safe 0 -i manifest.ffconcat -i chapters.txt -map_metadata 1 -map_chapters 1 -map 0:a:0 -c:a copy out.mp3` produces real ID3v2 `CHAP`/`CTOC` frames (confirmed via `ffprobe -show_chapters` and by grepping the output file for the `CHAP`/`CTOC` frame IDs) without a second pass or re-encode. Alternative considered: a separate `ffmpeg -i trimmed.mp3 -i chapters.txt -map_metadata 1 -c copy out2.mp3` post-processing pass — rejected because it doubles I/O and file churn for no benefit, since the concat command already accepts a second input for metadata mapping.

**No chapters embedded in the original `{yt_id}.mp3`.** Only the derived file changes. This keeps the original file's role (source of truth for trimming, and the file the web player streams) exactly as it is today.

## Risks / Trade-offs

- [Risk] yt-dlp's `chapters` field shape could change across versions (e.g., a video with a single implicit chapter, or missing `end_time` on the last entry) → Mitigation: parse defensively (`Option`/default for `end_time`, fall back to episode duration for the final chapter, ignore chapters with `start_time >= end_time`).
- [Risk] Titles may contain characters that need escaping in FFMETADATA1 (`;`, `#`, `\`, newlines) → Mitigation: escape per the FFMETADATA1 format (backslash-escape `=`, `;`, `#`, `\`, and newline) when writing the sidecar file.
- [Risk] A chapter's boundary sitting exactly at a retained-interval edge is an ambiguous case for the step function (`t == interval.start` vs `t == interval.end` for adjacent intervals) → Mitigation: the function's `<=` ordering (check `interval.start` before `interval.end`) makes it well-defined and consistent for both a chapter's start and end applied independently.
- [Trade-off] Storing chapters as JSON text (not a normalized table) means no per-chapter SQL querying, but that is never needed — episodes always read/write their full chapter list at once, matching the existing `sponsorblock_cache.segments_json` precedent.

## Migration Plan

- Add a migration creating the `chapters_json` column on `episodes` (nullable, defaulting to absent/empty for existing rows — no backfill possible since original yt-dlp chapter data for already-downloaded episodes was never retained).
- No changes required to already-downloaded episodes or already-generated derived MP3s; they simply have no chapters until re-downloaded (out of scope — episodes are not re-downloaded by this change).
- Existing derived MP3s remain valid; a future SponsorBlock reconciliation (triggered by a processing-hash change) will regenerate them with chapters if the episode has stored chapters by then.
