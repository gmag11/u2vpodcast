## 1. Cookie parity for the video listing

- [x] 1.1 In `Ytdlp::get_latest`, append `--cookies <file>` when `self.cookies` is non-empty (matching `download`)
- [x] 1.2 Extract a small shared flag helper so the download and listing paths cannot diverge again

## 2. Robust metadata parsing

- [x] 2.1 Rewrite `get_metadata` (`src/models/ytinfo.rs`) to match both `property`-before-`content` and `content`-before-`property` orderings, single or double quotes, with an optional extra-attribute gap between them
- [x] 2.2 Rewrite `get_image` with the same robustness, keeping the existing `?`-suffix truncation on the content group
- [x] 2.3 Compile the per-key `Regex` once (e.g. a `Lazy`/`OnceLock` registry) instead of constructing it per call

## 3. Tests

- [x] 3.1 Parser unit tests: canonical order (double quotes) → current behavior preserved
- [x] 3.2 Reversed order (`content` before `property`)
- [x] 3.3 Single-quoted attributes
- [x] 3.4 Extra attribute between `property` and `content`
- [x] 3.5 Missing meta → empty string; malformed/unclosed quote → empty string (no panic)
- [x] 3.6 Image `?`-suffix truncation still yields the clean URL

## 4. Verification

- [x] 4.1 Manual: create a channel from a real YouTube URL — non-empty title/description/image
- [x] 4.2 Manual: image refresh updates the stored image URL
- [x] 4.3 Confirm a listing for a cookie-protected channel matches what downloads can fetch (no silent omission)
- [x] 4.4 `cargo test` suite passes