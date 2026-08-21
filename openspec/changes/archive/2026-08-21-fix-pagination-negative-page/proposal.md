## Why

`User::read_with_pagination` computes `offset = (page - 1) * per_page` with an unvalidated `page` (`page.page.unwrap_or(1)`). `page = 0` or negative values produce a negative SQL OFFSET that SQLite rejects, surfacing as an avoidable HTTP 500. The same unsafe pattern exists in `Channel::read_with_pagination` and `Episode::read_with_pagination`.

## What Changes

- Clamp `page` to a minimum of 1 wherever the offset is computed (users, channels, episodes), so `0`/negative pages behave as page 1 instead of 500.
- Optionally reject absurdly large pages at the existing per_page cap level (only if trivially consistent).

## Capabilities

### New Capabilities

- `api-pagination`: Defines predictable, validated pagination semantics shared by the paginated endpoints.

### Modified Capabilities

(none)

## Impact

- `src/handlers/users.rs` (`read_with_pagination`), `src/models/user.rs`, `src/models/channel.rs`, `src/models/episode.rs`.
- No frontend change (the SPA already sends valid pages).
- Regression guard: re-analysis against `docs/bug-review-2026-08-21.md`; clamp must not shift results for valid page values (e.g. negative → page 1 exactly, not arbitrary offsets).