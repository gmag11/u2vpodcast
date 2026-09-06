## Why

Once the persistent player auto-hides after a stop, there is no way to bring the bar back without starting playback again. The user may want to inspect the stopped episode or the persisted up-next queue, or restart playback, without re-launching audio.

A stopped player currently keeps the bar visible while a queue is loaded but no current episode exists (the "queue-only" case), even though nothing is playing. The bar should hide in every stopped state — with or without a current episode, and regardless of the queue — so the reopened state is uniform and only reachable through an explicit control.

## What Changes

- Extend the bar's auto-hide so that every stopped state hides after its delay, including the queue-only case (restored queue without a current episode). A stopped bar with no current episode and an empty queue remains hidden (nothing to restore).
- Add a small floating "reopen player" control that appears only while the persistent player bar is hidden and there is restorable content (a current episode or a non-empty queue).
- Activating the control restores the persistent player bar without triggering playback, preserving the current episode, the queue, and the stopped state.
- A reopened stopped bar re-arms the auto-hide timer, so it hides again after the usual delay exactly as any other stop.
- The control works in both the desktop (wide) and mobile (compact + expanded) compositions.
- When the page loads with a restored queue but no active playback, the bar is hidden and the reopen control offers the only path back to the queue without playing.

## Capabilities

### New Capabilities
<!-- Capabilities being introduced. Replace <name> with kebab-case identifier (e.g., user-auth, data-export, api-rate-limiting). Each creates specs/<name>/spec.md -->

### Modified Capabilities
<!-- Existing capabilities whose REQUIREMENTS are changing (not just implementation).
     Only list here if spec-level behavior changes (not just implementation).
     Each needs a delta spec file.
     Use existing spec names from openspec/specs/. Leave empty if no requirement changes. -->
- `persistent-audio-player`: extend the stopped auto-hide to cover the queue-only case (no current episode, queue still loaded), and add a floating reopen control shown while the bar is hidden that restores the bar without starting playback.

## Impact

- Frontend only: `frontend/src/components/PersistentPlayer.vue` — the visibility watch logic (drop the "queue-only → keep visible" branch, or route it through the same auto-hide timer) plus a new floating control in the same component.
- No player-store change required: `visible` is component-local; the store already keeps `currentEpisode`/`upNext` intact across a stop.
- Component tests for the persistent player (auto-hide in queue-only state, reopen control lifecycle); spec delta for `persistent-audio-player`.
