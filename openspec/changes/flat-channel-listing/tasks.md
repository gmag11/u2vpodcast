## 1. Flat Listing

- [x] 1.1 Switch `Ytdlp::get_latest` to `--flat-playlist` (keeping `--dateafter`, `--break-on-reject`, cookies and throttle); `YtVideo` is serde-default tolerant of omitted flat fields
- [x] 1.2 Per-video metadata source: `Ytdlp::download` gains `--print-json`, captures stdout, and returns `(ExitStatus, YtVideo)` with the parsed info dict (same line-JSON parsing, throttled)

## 2. Worker Flow

- [x] 2.1 Worker filters flat candidates Rust-side by the `last` boundary (`filter_by_window` backstop) so out-of-window videos never reach a per-video connection; already-stored videos are skipped as today
- [x] 2.2 Complete episode rows are built from the download run's metadata (`--print-json`), falling back to the flat candidate for omitted fields; per-episode delay, throttle slot, and failure semantics unchanged

## 3. Verification & Regression

- [x] 3.1 Integration test with a fake `yt-dlp`: synthetic 300-entry flat listing spanning the window — parses all entries; the backstop keeps exactly the 50 in-window candidates (no per-video work for the 250 out-of-window)
- [x] 3.2 Unit tests: flat-listing parse tolerant of missing fields, `--print-json` full metadata parse, backstop boundary behavior (on-edge/older/no-date)
- [x] 3.3 Full test suite green (63 tests, 0 warnings). Live spot-check on a large channel (e.g. Defected Music) pending redeploy: listing should complete in seconds and per-video downloads resume immediately with metadata