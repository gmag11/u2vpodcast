## ADDED Requirements

### Requirement: Playlist order is edited with inline drag handles

When the playlist contains at least two episodes, each episode row in the main playlist view SHALL expose one compact six-dot drag handle immediately to the left of its existing episode card. The handle SHALL replace the stacked up/down buttons; reordering SHALL NOT require opening a separate drawer, modal, or editing mode. Dragging the handle SHALL move the complete episode row while preserving the card's playback and episode actions.

The user SHALL be able to move a row directly to any insertion position between the other rows with pointer or touch input. While a row is moving, the playlist SHALL visually identify the active row and the prospective insertion position between cards. While an episode is dragged near the top or bottom edge of the visible playlist, the page SHALL automatically scroll in that direction for as long as the pointer remains in the edge zone and further content is available. Auto-scroll SHALL stop when the pointer leaves the edge zone or the page reaches its boundary, allowing the episode to be dropped at a position that was not initially visible.

The handle SHALL provide an equivalent keyboard interaction that lets a focused row be picked up, moved, and dropped, and SHALL announce the operation state and resulting position to assistive technology.

On mobile viewports, the drag handle SHALL provide a touch target of at least 44 by 44 CSS pixels without materially narrowing or overlapping the episode card. Dragging SHALL start only from the handle so touching or swiping the card elsewhere preserves its existing controls and normal page scrolling.

Completing a drop that changes the order SHALL submit the complete resulting list of episode ids for persistence. The main playlist SHALL retain the new order when persistence succeeds. If persistence fails, the main playlist SHALL restore the last persisted order and the user SHALL receive an error notification. Dropping an episode without changing its position SHALL NOT submit a reorder request.

#### Scenario: Playlist presents compact drag handles
- **WHEN** the playlist contains at least two episodes
- **THEN** each episode card has one six-dot drag handle on its left and no up/down reorder buttons are shown

#### Scenario: Dropping an episode between two others
- **WHEN** the user drags an episode row by its handle and drops it between two other cards
- **THEN** the main playlist submits the complete resulting episode-id order and displays that order after persistence succeeds

#### Scenario: Dragging to an initially hidden position
- **WHEN** the user keeps a dragged episode near the top or bottom edge of the visible playlist and more episodes exist beyond that viewport edge
- **THEN** the page scrolls in that direction until the pointer leaves the edge zone or the page boundary is reached, exposing additional insertion positions without ending the drag

#### Scenario: Reordering on a mobile viewport
- **WHEN** the playlist is viewed on a narrow touch device
- **THEN** every drag handle has a touch target of at least 44 by 44 CSS pixels and neither overlaps nor causes horizontal overflow in its episode card

#### Scenario: Using the card without dragging
- **WHEN** the user clicks, taps, or swipes an episode card outside its drag handle
- **THEN** the card's existing actions and normal page scrolling remain available and no drag operation starts

#### Scenario: Reordering with a keyboard
- **WHEN** a keyboard user picks up a focused episode, moves it to a different insertion position, and drops it
- **THEN** the same complete order is persisted and the updated position is announced to assistive technology

#### Scenario: Reorder persistence fails
- **WHEN** the user completes a changed drop and the reorder request fails
- **THEN** the main playlist restores the last persisted order and an error notification is shown

#### Scenario: Episode is dropped in its original position
- **WHEN** the user drops an episode without changing its position
- **THEN** the visible order remains unchanged and no reorder request is sent

#### Scenario: Playlist cannot be meaningfully reordered
- **WHEN** the playlist contains fewer than two episodes
- **THEN** the playlist view does not expose drag handles or stepwise reorder controls