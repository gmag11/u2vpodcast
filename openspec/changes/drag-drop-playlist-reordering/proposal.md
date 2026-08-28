## Why

Reordering the playlist one position at a time with separate up/down buttons is slow and gives each card a visually heavy control column. A single drag handle beside each episode lets users place the whole row directly between any two episodes, matching the familiar compact grip pattern used by mobile playlist apps.

## What Changes

- Replace each episode's stacked up/down buttons with one compact six-dot drag handle at the left of the existing card.
- Reorder the existing playlist rows directly in the main view, moving the complete episode card and showing a visible insertion position between cards.
- Auto-scroll the page when a dragged row reaches the top or bottom edge of the visible playlist so off-screen positions remain reachable.
- Persist each completed drop through the existing full-order playlist reorder operation and restore the previous order if persistence fails.
- Make the handle usable with pointer, touch, and keyboard input, with a touch-sized mobile target, localized accessible labels, and status feedback.

## Capabilities

### New Capabilities
<!-- None. -->

### Modified Capabilities
- `playlist`: Replace stepwise playlist reordering controls with an inline drag handle beside each episode card that persists the resulting full order and remains touch and keyboard accessible.

## Impact

- Frontend playlist view and an inline draggable-row component or directive.
- Playlist store error handling for optimistic reorder and rollback.
- English and Spanish playlist translations and focused frontend tests.
- A frontend drag-and-drop dependency may be added; the backend, database schema, and existing playlist API contract remain unchanged.