## Why

Once the persistent player auto-hides after a stop, there is no way to bring the bar back without starting playback again. The user may want to inspect the current episode or the up-next queue, or restart playback, without re-launching audio.

## What Changes

- Add a small floating "reopen player" control that appears only while the persistent player bar is hidden.
- Activating the control expands/restores the persistent player bar without triggering playback.
- The control works in both the desktop (wide) and mobile (compact + expanded) compositions.
- The auto-hide behavior is preserved: if the player is stopped, the bar still hides after its timer, regardless of whether the control is shown.
- A hidden-but-restorable player must keep its state (current episode, queue) intact so reopening shows the same content.

## Capabilities

### New Capabilities
<!-- Capabilities being introduced. Replace <name> with kebab-case identifier (e.g., user-auth, data-export, api-rate-limiting). Each creates specs/<name>/spec.md -->

### Modified Capabilities
<!-- Existing capabilities whose REQUIREMENTS are changing (not just implementation).
     Only list here if spec-level behavior changes (not just implementation).
     Each needs a delta spec file.
     Use existing spec names from openspec/specs/. Leave empty if no requirement changes. -->
- `persistent-audio-player`: add a floating reopen control shown while the bar is auto-hidden, and require it to restore the bar without starting playback.

## Impact

- Frontend only: `frontend/src/components/PersistentPlayer.vue` (and possibly a new small control component).
- Player store (`frontend/src/stores/player.ts`) gains an action to reveal the bar without calling `togglePlay`/`play`; no state reset involved.
- Component tests for the persistent player; spec delta for `persistent-audio-player`.
