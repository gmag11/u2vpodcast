## 1. Configuration Contract

- [ ] 1.1 Add default-off `sponsorblock_enabled`, the supported category set, and `sponsorblock_rejected_categories` with its `sponsor` default, duplicate normalization, explicit empty-list handling, and invalid-value startup errors; verify focused Rust tests cover omitted, disabled, enabled, empty, duplicate, and invalid configurations.
- [ ] 1.2 Gate automatic synchronization and authenticated refresh before SponsorBlock retrieval or reconciliation, and thread the validated category selection only when enabled; verify disabled-path tests observe no client calls or cache/media mutations.

## 2. Categorized Snapshot Model

- [ ] 2.1 Add an up/down migration for the processing hash, backfill existing rows from `snapshot_hash`, and update cache reads/writes; verify migration tests preserve an existing active filename and hash state.
- [ ] 2.2 Extend persisted SponsorBlock segments with category metadata and legacy `{start,end}` deserialization as `sponsor`; verify model tests read both legacy and categorized cache JSON.
- [ ] 2.3 Request every supported `skip` category and normalize categorized intervals deterministically without merging descriptive entries; verify request and normalization tests reject unsupported categories/actions and preserve mixed-category overlaps.
- [ ] 2.4 Implement separate snapshot hashing and rejected-interval projection/processing hashing; verify tests cover configuration order and duplicates, cross-category overlap unions, playable-only changes, rejected changes, and an empty rejected selection.

## 3. Reconciliation And API

- [ ] 3.1 Update reconciliation and derived-media naming to use merged rejected intervals and the processing hash while persisting all categorized segments and their snapshot hash; verify playable-only changes avoid FFmpeg, rejected changes replace media, an empty rejected set restores original media, and the disabled path bypasses processing.
- [ ] 3.2 Expose `{start,end,category,rejected}` segments using current application configuration only when enabled; verify episode list, playlist, and refresh tests cover selected, unselected, empty-selection, legacy, disabled-data suppression, and disabled refresh without external calls.
- [ ] 3.3 Preserve the last valid snapshot and processed representation on retrieval or processing failures under the new dual-hash model; verify existing failure-path tests and new mixed-category failure cases pass.
- [ ] 3.4 Make channel and global feeds select original media and original duration whenever SponsorBlock is disabled, without deleting cached derivatives; verify both feed test suites cover enabled processed selection and disabled original fallback.

## 4. Frontend Playback And Markers

- [ ] 4.1 Extend frontend SponsorBlock types and snapshot application to retain category/rejection metadata, accept visible-segment changes without reloading the source, and clear SponsorBlock state when disabled; verify player-store tests cover identical, playable-only, rejected, and disabled snapshots.
- [ ] 4.2 Update skip-target calculation to union and skip only rejected overlapping intervals while allowing non-rejected intervals to play; verify playback, resume, scrubber, relative-seek, overlap, and empty-selection unit tests.
- [ ] 4.3 Make timeline markers include every segment and classify sponsor versus non-sponsor categories; verify marker-generation tests cover rejected and playable segments plus mixed-category overlaps.
- [ ] 4.4 Add a theme-aware non-sponsor marker color and render category-aware markers in episode cards and the persistent player only when enabled; verify component tests assert all enabled markers remain visible while idle/paused, use the expected two color classes, and disappear with the refresh action when disabled.

## 5. Operator Documentation

- [ ] 5.1 Add default-off `sponsorblock_enabled`, `sponsorblock_rejected_categories`, and adjacent comments to `config.yml`, documenting complete bypass, opt-in upgrade action, category defaults and values, empty-list behavior, restart/reconciliation timing, and enabled marker display; verify the checked-in sample parses successfully.
- [ ] 5.2 Update the README SponsorBlock section with both parameter contracts and clarify that disabled means no SponsorBlock functionality, while enabled cuts/skips only configured categories and shows every supported category; verify existing source and license attribution remains intact.

## 6. Integrated Verification

- [ ] 6.1 Run frontend formatting/linting, type checking, and the focused SponsorBlock component/store test suites; verify all commands complete successfully.
- [ ] 6.2 Run formatting plus focused Rust configuration, SponsorBlock model, reconciliation, handler, and migration tests, then run the full Rust test suite on Linux; verify all checks complete successfully without changing Unix-specific tests for Windows.
- [ ] 6.3 Exercise the playlist in desktop and mobile browser viewports with SponsorBlock disabled, then enabled with sponsor, non-sponsor rejected, and non-rejected segments; verify the disabled UI has no SponsorBlock controls, markers, or skips and the enabled UI renders correctly and skips only rejected intervals.