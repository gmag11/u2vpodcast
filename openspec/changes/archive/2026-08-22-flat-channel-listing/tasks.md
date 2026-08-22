## 1. Flat Bounded Listing

- [x] 1.1 `list_videos(url, count)` using `--flat-playlist` + `--playlist-items 1:<count>` (no `--dateafter`), cookies and throttle kept; `YtVideo` tolerates missing fields and gains optional `release_date` / `live_status` (serde defaults)
- [x] 1.2 `Ytdlp::download` with `--print-json` returning `(ExitStatus, YtVideo)` (line-JSON parse, throttled)

## 2. Count-Window Selection & Processing

- [x] 2.1 Candidate selection (`select_window`): candidates in listing order (the `/videos` tab is newest-first); `is_upcoming`/`is_live`/future-dated (1h tolerance) excluded; scan stops at the first entry older than the `first` floor; undated entries keep their listing position; window = first `max` (listing requested with `max + 5`)
- [x] 2.2 Per-video processing: skip stored, download missing; authoritative date re-checked against the `first` floor after download (discard file + no episode row when older); episode row built from `--print-json` metadata. Process logging: listing counts, exclusion/floor summary, per-episode progress (`Processing i/N`), discard messages

## 3. Verification & Regression

- [x] 3.1 Selection tests + integration with fake `yt-dlp` (300-entry flat listing): window = the 50 newest; `raising_max_includes_older_missing_episodes` covers the 20→30 acceptance
- [x] 3.2 Unit tests: listing-order window, upcoming/live/future exclusion, floor early-stop, undated keeps position, `--print-json` parse (tolerant), flat listing parse (tolerant)
- [x] 3.3 Full test suite green (67 tests, 0 warnings). Live spot-check on a large channel (e.g. Defected Music) pending redeploy: listing capped, gap-fill works, raising `max` pulls the older missing episodes