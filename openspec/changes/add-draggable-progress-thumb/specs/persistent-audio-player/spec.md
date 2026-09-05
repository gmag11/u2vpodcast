## ADDED Requirements

### Requirement: Compact composition provides interactive scrubbing
The compact (mobile) composition of the persistent player bar SHALL render an interactive progress track with a draggable thumb and drag-preview time tooltip, matching the behavior of the wide and expanded compositions, so users can scrub from the collapsed mobile player. The compact track SHALL no longer be read-only: it SHALL accept pointer drag and click to seek, subject to the same unknown-duration and SponsorBlock-skip rules as the other compositions.

#### Scenario: Compact track is scrubbable
- **WHEN** the compact composition is displayed and the user drags or clicks its progress track
- **THEN** the track seeks playback to the chosen position, with a draggable thumb and drag-preview tooltip shown as in the wide and expanded compositions

#### Scenario: Compact track honors unknown duration
- **WHEN** the current episode has zero or non-finite duration
- **THEN** the compact track's thumb is not draggable and seeking is not performed
