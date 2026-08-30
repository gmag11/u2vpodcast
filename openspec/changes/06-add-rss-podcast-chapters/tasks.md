## 1. Chapters JSON endpoint

- [ ] 1.1 Add a handler exposing `GET .../episodes/{yt_id}/chapters.json` (exact path per design.md's routing decision) returning `{ "chapters": [{ "startTime": <seconds>, "title": <string> }, ...] }`; verify with an integration test for an episode with stored chapters and the original enclosure selected
- [ ] 1.2 When the episode's active enclosure is a SponsorBlock-processed MP3, compute translated chapters via `01-add-chapter-capture-and-embed`'s `translate_chapters`/retained-intervals logic instead of the raw stored chapters; verify with an integration test comparing the endpoint's output against the same processed MP3's embedded ID3 chapters (via `ffprobe -show_chapters`) for equality
- [ ] 1.3 Verify an episode with no stored chapters returns `200 OK` with an empty `chapters` array (not `404`)

## 2. Feed XML

- [ ] 2.1 Register the Podcasting 2.0 `podcast` namespace on the channel builder in `src/handlers/feed.rs`; verify via a feed-generation test that the namespace declaration appears exactly once on the channel element
- [ ] 2.2 Add a `<podcast:chapters url="..." type="application/json+chapters"/>` extension element to each `<item>` for an episode with stored chapters, using that episode's chapters endpoint URL; verify via a feed-generation test that the URL matches the endpoint added in task 1.1
- [ ] 2.3 Verify an episode with no stored chapters produces an `<item>` with no `<podcast:chapters>` element

## 3. Regression and manual verification

- [ ] 3.1 Verify existing feed tests (ordering, enclosure selection, duration, legacy URLs) are unaffected
- [ ] 3.2 Manually validate a generated feed against a Podcasting 2.0-aware validator or client to confirm the namespace and element are well-formed and recognized
