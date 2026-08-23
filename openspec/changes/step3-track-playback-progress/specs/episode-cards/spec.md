## Purpose

Extends the episode card with playback progress indicators defined by the `playback-progress` capability: a visible played mark and a resume hint.

## ADDED Requirements

### Requirement: Played mark and resume hint on episode cards

The episode card SHALL render a played mark when the episode is marked listened, and a resume hint for partially played episodes, per the `playback-progress` capability.

#### Scenario: Played mark on completed episodes
- **WHEN** an episode has `listen` true
- **THEN** the card shows a visible played mark (check indicator with the "listened" label)

#### Scenario: Resume hint on partial episodes
- **WHEN** an episode has a stored position above 30 seconds and `listen` is false
- **THEN** the card shows a hint with the stored position (for example "Continue at MM:SS") and an affordance to start over

#### Scenario: No indicator for untouched episodes
- **WHEN** an episode has never been played or its position is at most 30 seconds
- **THEN** the card shows neither the played mark nor a resume hint