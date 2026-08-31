## 1. Chapters JSON endpoint

- [x] 1.1 Add a handler exposing `GET .../episodes/{yt_id}/chapters.json` (exact path per design.md's routing decision) returning `{ "version": "1.2.0", "chapters": [{ "startTime": <seconds>, "title": <string> }, ...] }` with content type `application/json+chapters`; verify with an integration test for an episode with stored chapters and the original enclosure selected
- [x] 1.2 When the episode's active enclosure is a SponsorBlock-processed MP3, compute translated chapters via `01-add-chapter-capture-and-embed`'s `translate_chapters`/retained-intervals logic instead of the raw stored chapters; verify with an integration test comparing the endpoint's output against the same processed MP3's embedded ID3 chapters (via `ffprobe -show_chapters`) for equality
- [x] 1.3 Verify an episode with no stored chapters returns `200 OK` with an empty `chapters` array (not `404`)
- [x] 1.4 Verify SponsorBlock enabled with an authoritative empty snapshot selects the original enclosure and returns the original, untranslated chapter times

## 2. Feed XML

- [x] 2.1 Register the Podcasting 2.0 `podcast` namespace on the channel builder in `src/handlers/feed.rs`; verify via a feed-generation test that the namespace declaration appears exactly once on the channel element
- [x] 2.2 Add a `<podcast:chapters url="..." type="application/json+chapters"/>` extension element to each `<item>` for an episode with stored chapters, using that episode's chapters endpoint URL; verify via a feed-generation test that the URL matches the endpoint added in task 1.1
- [x] 2.3 Verify an episode with no stored chapters produces an `<item>` with no `<podcast:chapters>` element

## 3. Regression and manual verification

- [x] 3.1 Verify existing feed tests (ordering, enclosure selection, duration, legacy URLs) are unaffected
- [x] 3.2 Manually validate a generated feed against a Podcasting 2.0-aware validator or client to confirm the namespace and element are well-formed and recognized

## 4. Original MP3 chapter preservation

- [x] 4.1 Request `--embed-chapters` in the yt-dlp MP3 download while retaining `--print-json` chapter capture for database persistence; verify with a download-arguments test
- [x] 4.2 Verify the full backend test suite and lint checks remain green
