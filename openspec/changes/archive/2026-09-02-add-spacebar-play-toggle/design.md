## Context

The player store owns the shared audio element, playback state, and a window-level `keydown` listener that currently handles left- and right-arrow seeking. The same handler already checks document focus, loaded-player availability, editable targets, and sliders. See `proposal.md` for motivation and `specs/persistent-audio-player/spec.md` for required behavior.

## Goals / Non-Goals

**Goals:**

- Keep global playback shortcuts in the existing player-store keyboard handler.
- Distinguish playing, paused, and stopped states before acting on spacebar input.
- Preserve native spacebar behavior for editable and interactive controls.
- Cover each state and exclusion with focused store tests.

**Non-Goals:**

- Changing arrow-key seek behavior or seek distance.
- Starting playback when no active paused session exists.
- Adding configurable shortcuts, visual shortcut hints, or backend behavior.

## Decisions

### Extend the existing window keydown handler

Handle the spacebar alongside the existing arrow keys in the player store. This keeps shortcut registration and lifecycle cleanup in one place and gives the shortcut direct access to the shared audio element and reactive playback state.

Alternative considered: register a separate component-level listener. Rejected because the shortcut is application-wide and component mounting could duplicate listeners or make behavior depend on the visible player composition.

### Gate spacebar behavior on explicit player state

Only invoke the existing play/pause behavior when an episode is loaded and `stopped` is false. Use the audio element's paused state to select resume or pause. This preserves the deliberate distinction between paused and stopped playback.

Alternative considered: call the generic toggle action whenever an episode is loaded. Rejected because that action could restart a stopped episode, violating the specification.

### Preserve native interaction before preventing defaults

Perform focus and target exclusions before calling `preventDefault()`. Keep existing editable and slider exclusions, and exclude native or role-based controls whose spacebar action must remain available. For accepted spacebar events, prevent the browser's page-scroll behavior before toggling playback.

Alternative considered: reuse only the arrow-key target filter unchanged. Rejected because spacebar activates controls such as buttons, while arrow keys generally do not.

## Risks / Trade-offs

- [Risk] Broad target exclusions could make the shortcut unavailable over some custom controls. Mitigation: exclude only editable elements and controls with native or declared spacebar interaction, with unit tests for representative targets.
- [Risk] Key auto-repeat could toggle playback repeatedly while spacebar is held. Mitigation: ignore repeated spacebar keydown events so one physical press produces one toggle.
- [Risk] Browser key values can differ between synthetic and legacy events. Mitigation: support the standard `event.key === ' '` value used by current browsers and project tests.

## Migration Plan

No data or deployment migration is required. Ship the frontend handler and tests together. Rollback consists of reverting the handler and its tests; persisted playback data remains compatible.
