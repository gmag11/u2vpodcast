## Context

**Cookies divergence:** `download` builds args, then `if !self.cookies.is_empty() { push("--cookies"); push(&self.cookies) }`. `get_latest` never does this. yt-dlp does attach cookies which affect which videos the flat playlist/`--dump-json` listing returns. For a channel configured with `cookies.txt` (there is a repo-level `cookies.txt`) the divergence means the listing sees fewer/other entries than downloads could handle. Because missing entries are simply invisible to the scan, there is no error surfaced — silent data loss.

**Fragile regex:** current patterns:
- `meta\s+property="og:title"\s+content="(?P<content>[^"]*)"`
- image: `meta\s+property="og:image"\s+content="(?P<content>[^"]*)"` then truncates at `?`.

Fail cases observed on real YouTube/og pages: `content="..." property="og:title"` (property clause later), single-quoted attributes, or `property="og:title" data-... content="..."`. Any deviation → `""`. Consequences: `Channel::new` writes an empty title/slug (later backfilled via `migrate_slugs` only if slug empty; title stays empty until operator edits), `update_image` writes an empty image URL, feed covers break.

## Goals / Non-Goals

**Goals:**
- Listings use the same credential set as downloads.
- Metadata regexes succeed on real og-format variations (both orders, both quote styles, extra attributes).

**Non-Goals:**
- No full HTML parsing library.
- No retry/semantics change for network failures.
- No re-fetch of existing channel metadata (operator-initiated refresh/new channels only).

## Decisions

- **Cookies:** mirror the download logic exactly — build a base `Vec` of args, conditionally append `--cookies`/file between the shared flags and the URL when `!self.cookies.is_empty()`. Factor a small helper to reduce the risk of the two call sites diverging again, e.g. `fn cookies_args(&self, args: &mut Vec<&str>)`.
- **Regex:** use a two-part capture that accepts either order and both quote styles.
  For each metadata key build `regex::Regex::new` once (not per call) — option order: `(?:content|property)\s*=\s*(['"])(?P<a1>.*?)\1 ... (?:content|property)\s*=\s*(['"])(?P<a2>.*?)\2` with backreference for the quote char and lazy `.*?`, then pick the capture belonging to `content`. For the image specifically, keep the existing `?`-suffix truncation applied to the content group.
  - Simpler robust alternative: two regexes tried in sequence — primary (property-before-content) and fallback (content-before-property) — matching single or double quotes via a char class `['"]` and capturing content with a lazy match. Choose this: it is easier to read and unit-test per pattern.
- **Lazy content capture across the rest of the tag:** ensure the content group stops at the first matching quote (lazy + same quote char) so an embedded space in the URL (rare for og:*) does not leak outside the attribute.
- **Tests:** unit tests in `ytinfo.rs` for: normal order double quotes (current format), reversed order, single quotes, extra attribute between, absent meta (empty string), `<` HTML entities as-is (no entity unescaping: out of scope).

## Risks / Trade-offs

- [YouTube changes markup again] → Regex remains inherently brittle; mitigation is tests + narrow pattern variants. If this recurs, the escape hatch is `yt-dlp --dump-json` for metadata (already available) — noted as future direction, not this change.
- [Cookies file presence changes listing scope] → Intended; parity with downloads corrects a data hole, it does not new-add risk (credentials already in use for downloads).

## Migration Plan

1. Patch `get_latest` args (cookies), extract shared flag builder.
2. Rewrite `get_metadata`/`get_image` parsers with fallback matching.
3. Add unit tests for the parser variants.
4. Manual: channel create + image refresh against a real channel URL; verify title/image non-empty; verify listing matches download capability for a cookie-protected channel.

## Open Questions

None.