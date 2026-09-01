## Why

Playback already supports global left- and right-arrow shortcuts, but users cannot toggle active playback from the keyboard without locating a player control. Adding a spacebar shortcut provides a familiar, efficient play/pause interaction while preserving the stopped state.

## What Changes

- Add a global spacebar shortcut that pauses a playing episode and resumes a paused episode.
- Ignore the spacebar when the player is stopped or no episode is loaded.
- Apply the same focus and editable-control exclusions used by the existing global seek shortcuts so normal page controls keep their native spacebar behavior.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `persistent-audio-player`: Define global spacebar play/pause behavior for active, paused, and stopped player states.

## Impact

- Frontend global keyboard handling in the player store.
- Player store unit tests for keyboard playback controls and focus exclusions.
- No backend, API, dependency, persistence, or versioning changes.
