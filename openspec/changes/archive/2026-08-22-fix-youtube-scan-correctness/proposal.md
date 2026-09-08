## Why

Two defects make the channel scan unreliable:

1. `Ytdlp::get_latest` (`src/models/ytdlp.rs:42`) never passes the configured cookies file, while `download` does. For channels with age-restricted or membership content, the listing can omit videos that the download step could then never reach — the scan and the download diverge, so episodes silently go missing.
2. The metadata extraction in `src/models/ytinfo.rs` (`get_metadata`, `get_image`) matches the HTML with fixed-order regexes expecting `property="..." content="..."` with double quotes. YouTube's HTML sometimes places `content` before `property`, uses single quotes, or adds attributes between them; the match silently yields an empty string, which then produces empty channel titles and generic fallback slugs (`channel-N`).

## What Changes

- `get_latest` forwards `--cookies <file>` when a cookies file is configured, matching the download path.
- The metadata regexes tolerate attribute order, both quote styles, and extra attributes between `property` and `content`.

## Capabilities

### New Capabilities

- `youtube-scan-reliability`: Defines that the video listing and the metadata used for channel titles/images are fetched with the same credentials as downloads and parsed robustly.

### Modified Capabilities

(none)

## Impact

- `src/models/ytdlp.rs` (args in `get_latest`), `src/models/ytinfo.rs` (parsing helpers).
- No API contract change; channel titles/slugs for channels with HTML ordering variations become correct on next create/update/refresh.

## Non-Goals

- No change to the download path (already cookie-aware).
- No change to the channel title/slug migration data (only new fetches benefit).
- No capture of a full HTML parser dependency; regex robustness is the scope.