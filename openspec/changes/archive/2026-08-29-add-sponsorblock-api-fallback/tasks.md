## 1. Endpoint Configuration

- [x] 1.1 Add the ordered primary and fallback SponsorBlock base URLs to the default client while preserving single-endpoint construction, and verify the Rust build reports no warnings for the client API.

## 2. Fallback Retrieval

- [x] 2.1 Iterate through configured SponsorBlock endpoints until one returns a usable snapshot, log fallback attempts, and verify successful primary responses remain preferred.
- [x] 2.2 Preserve HTTP 404 as an authoritative empty snapshot without fallback and return a retrieval error after all configured endpoints fail, verified by focused request tests.

## 3. Tests

- [x] 3.1 Add a local-server test where the primary connection is unavailable and the fallback returns segments, then run `cargo test -q utils::sponsorblock::tests::` and verify the SponsorBlock utility suite passes.
