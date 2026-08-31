## Context

`src/handlers/feed.rs` builds RSS via the `rss` crate's typed builders (`ChannelBuilder`, `ItemBuilder`) plus its `itunes` extension module for iTunes-specific tags. The crate also exposes a generic `rss::extension::Extension`/`ExtensionMap` mechanism for arbitrary namespaced elements, which is how a `<podcast:chapters>` element (Podcasting 2.0 namespace `https://podcastindex.org/namespace/1.0`) would be added without hand-rolling XML strings. `01-add-chapter-capture-and-embed` already computes translated chapters for the SponsorBlock-processed file at generation time (as part of embedding ID3 chapters); this proposal's JSON endpoint needs the same translated values, not a re-derivation from the audio file's embedded frames.

## Goals / Non-Goals

**Goals:**
- Keep the JSON endpoint's chapter times perfectly consistent with whichever enclosure (original vs processed) the feed actually references for that episode, reusing `01`'s translation function rather than re-parsing ID3 frames from the MP3.
- Use the `rss` crate's generic extension API for the namespace element rather than hand-rolled XML.

**Non-Goals:**
- Removing or replacing the ID3-embedded chapters from `01-add-chapter-capture-and-embed` — both mechanisms coexist.
- Any change to which enclosure is selected (existing `rss-feeds` capability logic is unchanged).

## Decisions

**Compute translated chapters for the active processed file rather than re-deriving them from ID3 frames.** `01-add-chapter-capture-and-embed`'s `translate_chapters` is a pure function over `(episode.chapters, retained_intervals)`; the feed handler can call it directly using the same episode/SponsorBlock-cache data it already loads (`sponsorblock_segments`, rejected categories) to reproduce the identical translated list, rather than parsing the derived MP3's ID3 chapter frames back out. This avoids adding an MP3-parsing dependency purely to read back data the backend already computed once.

**Add the chapters JSON endpoint under the existing episode/media routing area** (e.g., `GET /channels/{slug}/episodes/{yt_id}/chapters.json` or similar, following whatever path convention `src/handlers/episodes.rs` or `src/handlers/media.rs` already uses for per-episode, per-slug resources) rather than inventing a new top-level route family.

**Use `rss::extension::{Extension, ExtensionMap}` to build the `<podcast:chapters>` element**, registering the `podcast` namespace on the channel via `ChannelBuilder::namespaces(...)`, consistent with how the crate already registers the `itunes` namespace.

**Emit JSON Chapters version `1.2.0` and use the configured public base URL.** The current format requires a root `version` property and the `application/json+chapters` content type. The public standard requires HTTPS resource URLs, which deployments satisfy by configuring an HTTPS base URL; the application does not reject HTTP because local and private installations legitimately use it.

**Embed raw chapters during the original yt-dlp download.** Pass `--embed-chapters` to the existing extraction/conversion command while continuing to parse the same `--print-json` output for database persistence. This keeps the original MP3 and database chapter representations aligned without another FFmpeg pass. SponsorBlock processing remains separate and embeds translated chapters into its derived file.

## Risks / Trade-offs

- [Risk] The Podcasting 2.0 namespace URI or attribute names could be a point of subtle incompatibility with some clients if implemented from memory rather than the published spec → Mitigation: implementation must cross-check the exact namespace URI and JSON schema against the current Podcasting 2.0 namespace documentation before finalizing, not assume from this proposal alone.
- [Trade-off] Computing translated chapters at feed-generation time (rather than caching them) is cheap (pure arithmetic over a handful of chapters) and avoids a second source of truth, at the cost of redoing the same small computation on every feed request — acceptable given feed requests are infrequent relative to computation cost.
