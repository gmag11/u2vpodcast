## ADDED Requirements

### Requirement: Playlist order is edited in a drag-and-drop drawer

The playlist view SHALL expose a reorder command when at least two episodes are present. Activating it SHALL open a drawer containing every playlist episode in the current stored order as a compact draggable row. The main playlist SHALL NOT show per-episode stepwise reorder controls while this drawer-based editor is available.

The user SHALL be able to move a row directly to any insertion position between the other rows with pointer or touch input. While an episode is dragged near the top or bottom edge of the visible list, the list SHALL automatically scroll in that direction for as long as the pointer remains in the edge zone and further content is available. Auto-scroll SHALL stop when the pointer leaves the edge zone or the list reaches its boundary, allowing the episode to be dropped at a position that was not initially visible.

The drawer SHALL provide an equivalent keyboard interaction that lets a focused row be picked up, moved, and dropped, and SHALL announce the operation state and resulting position to assistive technology. While a row is being moved, the drawer SHALL visibly identify both the active row and its prospective insertion position.

On mobile viewports, the reorder editor SHALL render as a bottom sheet constrained to the available viewport height and device safe areas. Its header and close action SHALL remain reachable while the episode list scrolls independently. Drag handles and other interactive controls SHALL provide touch targets of at least 44 by 44 CSS pixels, episode text SHALL truncate rather than widen the sheet, and touching a row outside its drag handle SHALL preserve normal list scrolling.

Completing a drop that changes the order SHALL submit the complete resulting list of episode ids for persistence. The drawer SHALL reflect the new order when persistence succeeds. If persistence fails, the drawer and the main playlist SHALL restore the last persisted order and the user SHALL receive an error notification. Dropping an episode without changing its position SHALL NOT submit a reorder request.

#### Scenario: Opening the reorder drawer
- **WHEN** the playlist contains at least two episodes and the user activates the reorder command
- **THEN** a drawer opens with all playlist episodes in their current stored order and each episode has a drag handle

#### Scenario: Dropping an episode between two others
- **WHEN** the user drags an episode and drops it between two other episodes
- **THEN** the drawer submits the complete resulting episode-id order and displays that order after persistence succeeds

#### Scenario: Dragging to an initially hidden position
- **WHEN** the user keeps a dragged episode near the top or bottom edge of a scrollable playlist and more episodes exist beyond that visible edge
- **THEN** the list scrolls in that direction until the pointer leaves the edge zone or the list boundary is reached, exposing additional insertion positions without ending the drag

#### Scenario: Reordering on a mobile viewport
- **WHEN** the user opens the reorder editor on a narrow touch device with a display cutout or home indicator
- **THEN** it appears as a bottom sheet within the visible safe area, keeps its header and close action reachable, provides touch-sized drag handles, and scrolls the episode list without horizontal overflow

#### Scenario: Scrolling the mobile list without dragging
- **WHEN** the user swipes vertically on an episode row outside its drag handle
- **THEN** the list scrolls normally and no drag operation starts

#### Scenario: Reordering with a keyboard
- **WHEN** a keyboard user picks up a focused episode, moves it to a different insertion position, and drops it
- **THEN** the same complete order is persisted and the updated position is announced to assistive technology

#### Scenario: Reorder persistence fails
- **WHEN** the user completes a changed drop and the reorder request fails
- **THEN** both the drawer and main playlist restore the last persisted order and an error notification is shown

#### Scenario: Episode is dropped in its original position
- **WHEN** the user drops an episode without changing its position
- **THEN** the visible order remains unchanged and no reorder request is sent

#### Scenario: Playlist cannot be meaningfully reordered
- **WHEN** the playlist contains fewer than two episodes
- **THEN** the playlist view does not expose the reorder command