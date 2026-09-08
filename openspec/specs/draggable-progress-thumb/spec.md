## ADDED Requirements

### Requirement: Progress track renders a draggable thumb at the playback position
Every interactive progress track of the persistent player SHALL render a visible draggable thumb (a dot) positioned at the current playback position along the track. The thumb SHALL use the same accent color as the progress fill so it reads as part of the progress. The thumb SHALL be large enough to target comfortably on touch and pointer input, and its hit area SHALL extend beyond the visible dot. The thumb SHALL be rendered on top of the track's SponsorBlock segment markers and chapter markers.

#### Scenario: Thumb shows at current position
- **WHEN** the persistent player is showing a progress track for an episode with elapsed playback
- **THEN** a draggable dot appears on the track at the position corresponding to the current playback time, colored like the progress fill

#### Scenario: Thumb overlays markers
- **WHEN** the progress track also renders SponsorBlock segments or chapter markers
- **THEN** the thumb is drawn above them so it remains visible and targetable

### Requirement: Dragging the thumb previews a target position
When the user starts dragging the thumb, the SHALL player preview the drag without committing a seek: the thumb SHALL follow the pointer along the track, and a tooltip SHALL appear above the thumb showing the time (using the same `elapsed / total` label format as the player readout) of the position that would be sought if the control were released at that point. The tooltip SHALL update continuously as the thumb moves, and SHALL stay within the viewport.

#### Scenario: Tooltip shows target time while dragging
- **WHEN** the user drags the thumb to a position on the track
- **THEN** a tooltip above the thumb continuously shows the time corresponding to the current pointer position, and playback does not jump while dragging

#### Scenario: Tooltip clamps to viewport edges
- **WHEN** the thumb is dragged to either end of the track
- **THEN** the tooltip remains fully visible inside the viewport

### Requirement: Releasing the thumb seeks to the target
When the user releases the thumb after dragging, the SHALL player seek playback to the previewed position, subject to the existing rejected-interval skip behavior for SponsorBlock. Releasing without an effective drag (a simple press and release, i.e. a click) SHALL keep the existing click-to-seek behavior. When the episode duration is not known (zero or non-finite), the thumb SHALL NOT be draggable and no seek SHALL occur.

#### Scenario: Release seeks to the previewed position
- **WHEN** the user releases the thumb after dragging it to a new position
- **THEN** the shared player seeks to that position (with SponsorBlock skip applied) and the tooltip hides

#### Scenario: Simple click still seeks
- **WHEN** the user presses and releases on the track without meaningfully moving the thumb
- **THEN** the existing click-to-seek behavior applies and the tooltip does not persist

#### Scenario: Unknown duration disables dragging
- **WHEN** the current episode has zero or non-finite duration
- **THEN** the thumb is not draggable and seeking is not performed
