# Design: flat-channel-listing

## Context

The current sync is date-boundary based: `process_channel` computes `last` (newest stored episode, or `channel.first`), lists with `--dateafter`, and downloads everything on/after that date. Pruning or gaps make this fragile — raising `max` never recovers older episodes, and undated flat entries are mishandled (the old fallback ranked them as newest). This change replaces the boundary with a count-window: the `max` most recent videos are the candidates, ordered by published date, with upcoming/future entries excluded and `first` acting as a hard floor.

## Goals / Non-Goals

**Goals:**
- Candidate window = the `max` most recent videos (newest by date), independent of what was downloaded before.
- Raising `max` backfills the older missing episodes; shrinking it keeps the existing pruning behaviour.
- Upcoming/live/future-dated entries never selected; undated entries never displace dated ones and are validated at download.
- Flat, bounded listing (max + margin) so the per-cycle cost is constant.
- `first` as a hard floor; full metadata from the single download run; throttle preserved.

**Non-Goals:**
- No change to retention pruning (`clean_channel`), channel delete, image cache, or the public API.
- No per-channel priority, scheduling changes, or queue semantics beyond the window.

## Decisions

- **Flat bounded listing.** `list_videos_wanted(url, count)` runs `yt-dlp --flat-playlist --dump-json --playlist-items 1:<count>` (cookies + throttle as today). `--playlist-items` caps the pages yt-dlp walks (≈ the "month by month" walk in the user's description, minus the dateafter). `count = max + MARGIN` so exclusions (upcoming/failed/private) cannot starve the window.
  - `MARGIN` = 5 fixed (open question: proportional). A shortfall heals on the next cycle.
- **Selection.** The channel `/videos` tab orders by publish date, newest first, and the flat listing preserves it; therefore candidates are taken **in listing order** (the "n most recent" is the first `max` entries). Dates are used only for the exclusion rules and the `first` floor:
  - Exclusion: `live_status == "is_upcoming"` or `"is_live"` → drop; parsed date strictly in the future beyond the 1h tolerance → drop.
  - Floor: walking newest-first, the scan stops at the first candidate whose date is older than `first` (the rest are older still). Entries without a parseable date keep their listing position; the floor is enforced at download.
  - When no date is available at all for a candidate, its listing position decides; the authoritative date arrives with `--print-json` and is validated against the floor.
- **`first` = floor.** `process_channel` passes `channel.first` to the per-video step. After the download, if the authoritative published date is before the floor, the file is removed and the episode is not stored (this also resolves undated candidates that slipped through selection).
- **Single-run metadata.** `download` adds `--print-json` and returns `(ExitStatus, YtVideo)` from its stdout — one throttled connection per new episode. Undated flat candidates resolved here. Rejected: a separate per-video extraction pass.
- **Parsing.** `YtVideo` gains serde-default fields `release_date` and `live_status`; `parse_dump_output` (line-JSON) is reused for flat listings and `--print-json` output. The old `--dateafter`/`--break-on-reject` and the date-window backstop disappear (replaced by count selection + floor).

## Risks / Trade-offs

- [Fixed margin may be short with many upcoming/failed entries] → Next cycle re-selects and heals; proportional margin is a documented follow-up.
- [Newest-first assumption vs date ordering] → Selection is by parsed date, not listing order; a wrong listing order only costs listing size (`--playlist-items` cap), never wrong episodes.
- [`release_date`/`live_status` presence varies by yt-dlp version] → All optional (serde default); absence degrades to the undated path (ranked last, resolved at download).
- [Future-date tolerance too small/large] → 1h default covers clock skew; documented constant, easy to adjust.

## Migration Plan

1. `list_videos_wanted` (flat + `--playlist-items` cap); `YtVideo` tolerates and gains `release_date`/`live_status`.
2. `download` with `--print-json` returning `(ExitStatus, YtVideo)`.
3. Worker selection (date ordering, exclusions, floor) + per-video processing with floor re-check.
4. Tests: 20→30 acceptance with a fake `yt-dlp`; exclusion/ordering unit tests; full suite; live spot-check.
5. Rollback: date-boundary behaviour resumes by reverting the listing/worker changes; no migration.

## Open Questions

- Margin: fixed +5 vs proportional (e.g. 20%) — fixed chosen; revisit if cycles report persistent shortfall.
- Whether `is_live` entries should be excluded (decided: yes, no usable audio until the stream ends) or skipped only on failure stay.
- `--playlist-items` semantics on YouTube channel tabs: verified during implementation; if unsupported, fall back to flat listing + Rust-side truncation (same bounded cost).