## 1. Flat Bounded Listing

- [ ] 1.1 `get_latest` becomes `list_videos_wanted(url, count)` using `--flat-playlist` + `--playlist-items 1:<count>` (no `--dateafter`), keeping cookies and the throttle; `YtVideo` tolerates missing fields and gains optional `release_date` / `live_status` (serde defaults)
- [ ] 1.2 `Ytdlp::download` gains `--print-json`, captures stdout, and returns `(ExitStatus, YtVideo)` with the parsed info dict (same line-JSON parsing, throttled)

## 2. Count-Window Selection & Processing

- [ ] 2.1 Candidate selection in the worker: take candidates **in listing order** (the `/videos` tab is newest-first, flat listing preserves it); exclude `is_upcoming`, `is_live`, and future-dated (1h tolerance) entries; stop the scan at the first entry older than the `first` floor (undated entries keep their listing position, floor enforced at download); the window is the first `max` candidates (listing requested with `max + MARGIN`)
- [ ] 2.2 Per-video processing: skip already-stored episodes; download missing ones; after download re-check the authoritative date against the `first` floor and discard (file removed, no episode row) when older; build the episode row from the `--print-json` metadata otherwise

## 3. Verification & Regression

- [ ] 3.1 Acceptance test (20→30) with a fake `yt-dlp`: synthetic channel (~35 entries: dated across the window + one upcoming + one future-dated + undated) — with `max=20` only the missing among the 20 newest dated are downloaded; raising to `max=30` downloads the 10 older missing ones; upcoming/future never downloaded
- [ ] 3.2 Unit tests: date scoring/ordering (undated last, future and `is_upcoming`/`is_live` excluded, deterministic tiebreak), floor filtering, `--print-json` metadata parse (tolerant)
- [ ] 3.3 Run the full test suite; live spot-check a large channel (e.g. Defected Music): listing capped, gap-fill works, and raising `max` pulls the older missing episodes