## Why

`UpdateChannel.max: i64` is accepted with no validation. Two severe consequences in `clean_channel` (worker): with `max = 0` a single sync deletes **every** mp3 and episode row of the channel (unrecoverable media loss via one API call); with a negative `max`, `usize::try_from` errors every cycle, so the channel is permanently marked `last_sync_ok=false`, never prunes, and accumulates episodes. The DB default is even `-1`.

## What Changes

- Server-side validation: `max` SHALL be `>= 1` on channel create and update; invalid values are rejected with a clear error and a 4xx response.
- Defense-in-depth: `clean_channel` SHALL never delete anything when `max` is invalid or below the safe minimum.
- The edit dialog SHALL clamp its `max` input to `>= 1`.

## Capabilities

### New Capabilities

- `channel-retention-limit`: Defines validation of the per-channel episode retention limit (`max`) that drives pruning.

### Modified Capabilities

(none)

## Impact

- `src/models/channel.rs` (validation on create/update), `src/utils/worker.rs` (`clean_channel` guard), `src/handlers/channels.rs` (error mapping), `frontend/src/components/AddChannelDialog.vue` (clamp).
- No schema change.
- Regression guard: re-analysis against `docs/bug-review-2026-08-21.md`; no new bugs — validation must not affect existing channels with currently-stored `max` values in the DB that are below 1 (they must be handled safely, e.g. by the clean guard).