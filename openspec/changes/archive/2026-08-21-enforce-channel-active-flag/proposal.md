## Why

The UI exposes an "Active" toggle per channel and persists it (`Channel::update` writes `active`), but the worker (`do_the_work` → `Channel::read_all`) processes **every** channel with no `active` filter. Disabling a channel does nothing: it keeps syncing, downloading, consuming disk/bandwidth and accumulating episodes.

## What Changes

- The scheduled worker SHALL skip channels whose `active` flag is `false`.
- The `active` toggle in the UI therefore truly stops automatic syncing.
- Existing behavior for active channels is unchanged; manual refresh/operations remain possible regardless of the flag unless decided otherwise.

## Capabilities

### New Capabilities

- `channel-active-state`: Defines that the `active` flag controls inclusion in the scheduled sync worker.

### Modified Capabilities

(none)

## Impact

- `src/utils/worker.rs` (`do_the_work`), `src/models/channel.rs` (`read_all` — filter or separate query).
- UI change: none required (toggle already exists); its effect becomes real.
- Regression guard: re-analysis against `docs/bug-review-2026-08-21.md` after implementation; no new bugs (e.g. never-synced edge cases must keep working).