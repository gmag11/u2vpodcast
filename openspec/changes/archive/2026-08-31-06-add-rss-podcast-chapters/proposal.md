## Why

`01-add-chapter-capture-and-embed` embeds chapters into the SponsorBlock-derived MP3 as ID3v2 `CHAP`/`CTOC` frames, which not every podcast app reads. The Podcasting 2.0 `<podcast:chapters>` tag plus a JSON chapters endpoint is a more widely and increasingly supported, format-agnostic mechanism for exposing chapters to external podcast clients, and is complementary to (not a replacement for) the embedded ID3 frames.

## What Changes

- Add a JSON chapters endpoint per episode following the Podcasting 2.0 JSON Chapters 1.2 schema: an object with `version: "1.2.0"` and a `chapters` array of `{startTime, title}`, reachable per the feed's existing URL/routing conventions.
- Reference that endpoint from each feed `<item>` via a `<podcast:chapters url="..." type="application/json+chapters"/>` element, using the `podcast` XML namespace declared on the channel.
- The JSON exposed for a given episode SHALL match the timeline of that episode's *selected* enclosure (mirroring the existing "Feed duration matches the selected media representation" precedent): the original episode's untranslated chapters when the original MP3 is selected, or the same translated chapters embedded into the derived MP3 (via `01-add-chapter-capture-and-embed`'s translation function) when the processed MP3 is selected.
- An episode with no stored chapters SHALL have no `<podcast:chapters>` element and its chapters endpoint SHALL respond with an empty chapters array (not a 404), consistent with how an episode with no SponsorBlock segments still returns a valid (empty) snapshot.
- Build the chapters URL from the configured public base URL. Deployments using HTTPS produce the standards-required HTTPS URL, while HTTP remains supported for local and private installations.
- Preserve chapters in both representations: continue persisting yt-dlp's chapter JSON in the database and request that yt-dlp embed those chapters into the original MP3. SponsorBlock-derived MP3s continue to receive translated embedded chapters.

## Capabilities

### New Capabilities
- `rss-podcast-chapters`: the JSON chapters endpoint and its Podcasting-2.0 authoring in the feed XML.

### Modified Capabilities
- `episode-chapters`: original MP3 downloads embed the same raw chapters that are persisted in the database.

## Impact

- Backend: `src/handlers/feed.rs` (namespace declaration, `<podcast:chapters>` element per item), a new handler exposing the per-episode chapters JSON (likely alongside `src/handlers/media.rs` or `src/handlers/episodes.rs`), `src/models/ytdlp.rs` (embed chapters in original downloads), reuses the chapter-translation function introduced by `01-add-chapter-capture-and-embed`.
- Depends on `01-add-chapter-capture-and-embed` for both the raw `chapters` data and the translation function used to keep the JSON consistent with the embedded ID3 chapters.
