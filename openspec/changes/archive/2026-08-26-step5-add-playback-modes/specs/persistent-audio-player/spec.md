## Purpose

Extends the persistent bottom player bar with shuffle and repeat mode toggles defined by the `playback-modes` capability.

## ADDED Requirements

### Requirement: Shuffle and repeat toggles in the persistent bar

The persistent player bar SHALL expose shuffle and repeat controls reflecting the player's mode state, as specified by the `playback-modes` capability.

#### Scenario: Shuffle toggle visible and reactive
- **WHEN** the bar is visible
- **THEN** a shuffle control is shown and highlights when shuffle mode is active

#### Scenario: Repeat toggle cycles states
- **WHEN** the bar is visible
- **THEN** a repeat control is shown that cycles through none, all, and one, visually indicating the active state