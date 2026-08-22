# Design: flat-channel-listing

## Context

Today the worker's `process_channel` calls `Ytdlp::get_latest(url, last)` which runs `yt-dlp --dateafter <last> --dump-json ... <channel>` (no `--flat-playlist`). `--dump-json` without `--flat-playlist` forces full extraction of every video that yt-dlp touches, and `--dateafter` only knows a video's date after extracting it. On a 4000-video channel with a 2023 backfill window (~1000-2000 videos), the app spends tens of minutes silently holding the single throttle slot before `get_latest` returns; `Command::output()` buffers everything, so there is no progress signal at all.

Goal: make the listing cheap (`--flat-playlist`), and obtain complete metadata only for the videos we will actually download, from the download run itself.

## Goals / Non-Goals

**Goals:**
- Flat, page-bounded listing for any channel size.
- Per-video full metadata exactly once, from the download run (no separate extraction pass).
- Rust-side backstop for the date boundary so a flat-listing extractor gap cannot blow up the candidate set.
- Preserve current semantics: `first`/`last`, `max` retention, episode rows, throttle coverage, per-episode logging.

**Non-Goals:**
- No Caching/DB changes; episode schema unchanged.
- No changes to `max` pruning, channel delete, or image caching.
- No pagination API redesign of the throttle change itself.

## Decisions

- **Flat listing:** `get_latest` runs `yt-dlp --flat-playlist --dateafter <ymd> --break-on-reject --dump-json <channel>` through the existing `with_youtube_slot` + `parse_dump_output` pipeline. `--flat-playlist` requests only the channel API pages (id/title/url/timestamp), which for a thousands-video channel is seconds. `--break-on-reject` limits flat output to the in-window prefix when the extractor applies the date filter to flat entries.
  - **Backstop in code:** regardless of whether `--dateafter` filters flat entries, `get_latest` returns entries with whatever date fields are present plus the parsed `timestamp` when available. The worker filters candidates Rust-side (`timestamp >= window`) so out-of-window videos never reach per-video work even if the extractor does not filter flat entries.
  - If a YouTube flat entry lacks any usable date field, the candidate is conservatively kept (rare; per-video work decides from its own metadata and skips if the actual date is out of window). This keeps correctness at the cost of an occasional detail fetch — bounded by the `max`/retention limit in the worst case.
- **Single-run metadata (`--print-json`):** `Ytdlp::download` adds `--print-json` and captures stdout; the info dict is parsed with the same line-JSON approach as `parse_dump_output` to fill the `YtVideo` (title, description, duration, thumbnail, upload date, id). This makes each new episode exactly one throttled connection: download + metadata in the same run. If the download fails, no json is emitted and the episode is not stored (unchanged semantics).
  - Rejected: a separate `get_video_info` extraction pass per video — an extra throttled connection per new video and slower backfills.
- **Worker flow (unchanged boundaries):** `process_channel` keeps computing `last` as today, calls the flat `get_latest`, and for each candidate: skip if already stored (`episode_exists`), skip if backstop-filtered out of window, otherwise `download` + parse metadata + `Episode::new`. Per-episode delay and throttle behavior unchanged.
- **Parsing reuse:** `parse_dump_output` already handles newline-delimited JSON objects; flat and `--print-json` outputs use the same shape. `YtVideo` gets serde defaults for fields that flat listings may omit.

## Risks / Trade-offs

- [Flat entries may not carry a parseable date in every extractor version] → Backstop keeps the candidate and the per-video metadata decides; worst case one extra detail per candidate, bounded by the window/retention size. If this becomes common, the follow-up is a separate bounded detail pass.
- [`--print-json` output reliability with `-x`/`-o`] → Parses the line JSON defensively; a missing/empty episode just errors that run (skipped, logged), same as today's parse failures.
- [`--break-on-reject` ordering assumption (newest-first)] → Filtering is also done Rust-side against the `last` boundary, so a wrong ordering only costs listing output size, never wrong episodes.
- [Larger flat output for very old windows] → Listing pays pages, not extraction; the candidate set is then filtered Rust-side before any per-video connection.

## Migration Plan

1. Switch `get_latest` to flat listing; make `YtVideo` serde-default tolerant.
2. Add `--print-json` + stdout capture to `download`; parse the info dict.
3. Worker: backstop date filter on candidates; build complete episodes from the download metadata.
4. Verify: local fake-yt-dlp integration test with a synthetic 1000-entry flat listing asserting only in-window candidates hit download; full test suite; live spot-check on Defected Music.
5. Rollback: revert `get_latest`/`download`; old behavior returns without migration.

## Open Questions

- Does `--dateafter` filter YouTube flat-playlist entries in the installed yt-dlp (stable 2026.08.19)? The code backstop makes this an optimization, not a correctness requirement; verify during verification.
- Does `--print-json` always emit before `-x`/post-processing? The parse reads the first JSON line; if conversion reorders output, fall back to capturing `--print-json` into a separate call (documented fallback, not currently planned).