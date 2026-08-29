## Why

SponsorBlock retrieval currently depends on a single hostname, so DNS, network, or service failures at `sponsor.ajay.app` prevent synchronization even when the official alternative endpoint remains available. Adding an ordered fallback improves retrieval resilience without changing the stored snapshot contract.

## What Changes

- Query `sponsor.ajay.app` as the primary SponsorBlock endpoint.
- Retry retrieval through `api.sponsor.ajay.app` when the primary request or response cannot be used.
- Continue treating a valid no-segments response, including HTTP 404, as an authoritative empty snapshot without querying the fallback.
- Preserve existing failure handling when neither endpoint yields a usable response.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `sponsorblock-integration`: Define ordered fallback behavior for SponsorBlock segment retrieval.

## Impact

- Affected code: the Rust SponsorBlock HTTP client in `src/utils/sponsorblock.rs` and its unit tests.
- External systems: `sponsor.ajay.app` remains primary; `api.sponsor.ajay.app` becomes the fallback.
- APIs and dependencies: no public API, configuration, database, frontend, or dependency changes.
