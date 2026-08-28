## Why

Reordering the playlist one position at a time with up/down buttons is slow and obscures the intended final order. A dedicated drawer with drag and drop lets users move an episode directly between any two episodes while keeping the main playlist focused on playback.

## What Changes

- Replace the per-episode up/down reorder controls in the main playlist with a reorder command that opens a dedicated drawer.
- Show the full playlist as compact draggable rows inside the drawer, with a visible insertion position and edge-triggered auto-scroll while dragging.
- Persist each completed drop through the existing full-order playlist reorder operation and restore the previous order if persistence fails.
- Make reordering usable with pointer, touch, and keyboard input, with a mobile bottom-sheet layout, touch-sized controls, safe-area spacing, localized accessible labels, and status feedback.

## Capabilities

### New Capabilities
<!-- None. -->

### Modified Capabilities
- `playlist`: Replace stepwise playlist reordering controls with drawer-based drag-and-drop reordering that persists the resulting full order and remains keyboard accessible.

## Impact

- Frontend playlist view and a new or extracted reorder-drawer component.
- Playlist store error handling for optimistic reorder and rollback.
- English and Spanish playlist translations and focused frontend tests.
- A frontend drag-and-drop dependency may be added; the backend, database schema, and existing playlist API contract remain unchanged.