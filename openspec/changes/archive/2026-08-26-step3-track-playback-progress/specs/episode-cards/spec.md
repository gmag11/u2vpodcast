## Purpose

Extends the episode card with compact playback-progress indicators defined by the `playback-progress` capability: a corner played mark, a resume hint, and a read-only progress strip.

## ADDED Requirements

### Requirement: Played mark, resume hint, and progress strip on episode cards

The episode card SHALL render its playback state compactly, per the `playback-progress` capability: the top-right corner tinted green when the episode is listened (no label or icon), a resume hint for partially played episodes, and a read-only progress strip spanning the card's bottom edge that reflects the saved position (the live playhead for the currently playing episode) and ignores pointer interaction.

#### Scenario: Played mark on completed episodes
- **WHEN** an episode has `listen` true
- **THEN** the card's top-right corner is tinted green instead of showing a label or check

#### Scenario: Resume hint on partial episodes
- **WHEN** an episode has a stored position above 30 seconds and `listen` is false
- **THEN** the card shows a hint with the stored position (for example "Continue at MM:SS") and an affordance to start over

#### Scenario: Progress strip reflects the saved point
- **WHEN** an episode has a saved position
- **THEN** the card shows a bottom progress strip sized proportionally to `position_seconds` over the episode duration

#### Scenario: Progress strip is read-only
- **WHEN** the user clicks or drags on the card's progress strip
- **THEN** playback is unaffected (the strip has no interaction handlers)

#### Scenario: No indicator for untouched episodes
- **WHEN** an episode has never been played or its position is at most 30 seconds
- **THEN** the card shows neither the played mark nor a resume hint, and no progress strip