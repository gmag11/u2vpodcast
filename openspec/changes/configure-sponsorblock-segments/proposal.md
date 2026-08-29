## Why

SponsorBlock processing currently rejects only `sponsor` segments, so operators cannot choose other SponsorBlock categories that they do not want in downloaded feeds or web playback. The UI also cannot distinguish sponsor segments from other reported categories or show categories that remain playable.

## What Changes

- **BREAKING** Add a documented `sponsorblock_enabled` master switch, defaulting to `false`, that bypasses all SponsorBlock retrieval, processing, API, playback, refresh, marker, and feed-selection behavior when disabled; existing operators must set it to `true` to retain current SponsorBlock behavior.
- Add a documented `config.yml` parameter that selects which supported SponsorBlock categories are rejected, defaulting to `sponsor` for backward compatibility.
- Retrieve, normalize, persist, and expose all supported `skip` segments while retaining enough category and rejection information to separate display from processing behavior.
- Generate derived audio and perform web-player skipping only for segments whose categories are selected for rejection.
- Display every available SponsorBlock segment on episode-card and persistent-player progress tracks, whether rejected or playable.
- Keep the existing sponsor marker color and introduce a distinct shared color for all non-sponsor categories.
- Document the parameter, its default, supported values, and processing/display semantics in both `config.yml` and the README.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sponsorblock-integration`: Add a default-off master enable switch, make rejected categories configurable, retain all supported segment categories in snapshots and episode payloads, and document the configuration contract.
- `persistent-audio-player`: Omit SponsorBlock playback and marker behavior while disabled; while enabled, skip only configured rejected segments and render all categories with sponsor and non-sponsor marker colors.
- `rss-feeds`: Select original media unconditionally while SponsorBlock is disabled.
- `global-feed`: Select original media and duration unconditionally while SponsorBlock is disabled.

## Impact

- Backend configuration loading and validation, SponsorBlock requests, normalization and hashing, snapshot persistence, media processing, synchronization, and refresh behavior.
- Episode API segment shape gains category and rejection metadata consumed by the frontend.
- Frontend episode types, player skip logic, marker generation, episode cards, persistent player, theme colors, and tests.
- Operator-facing `config.yml` and README documentation for both SponsorBlock parameters.
- Existing stored snapshots may require refresh or compatibility handling to populate category metadata without interrupting current media selection.