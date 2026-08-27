# episode-cards

## Purpose

Defines the layout and inline playback controls of the episode card in the Vue 3 SPA. Cards keep a compact vertical footprint by exposing only play/pause and stop bound to the shared player, showing a total-duration-only label, and placing the controls per breakpoint; advanced controls (seek, volume, speed) live exclusively in the persistent bottom player bar.

## ADDED Requirements

### Requirement: Favorite toggle on episode cards

Each episode card SHALL expose a favorite toggle rendered as a star icon reflecting the episode's stored favorite flag: a hollow (outline) star SHALL indicate a non-favorite and a filled star SHALL indicate a favorite. Activating the toggle on a non-favorite marks the episode as favorite; activating it on a favorite unmarks it, with a notification on each action. The toggle's state SHALL come from the episode's `favorite` field and SHALL update immediately when changed, without a list refetch.

#### Scenario: Non-favorite episodes show a hollow star
- **WHEN** an episode's `favorite` is false and the card is rendered
- **THEN** the card shows the favorite toggle as a hollow (outline) star icon

#### Scenario: Favorite episodes show a filled star
- **WHEN** an episode's `favorite` is true and the card is rendered
- **THEN** the card shows the favorite toggle as a filled star icon

#### Scenario: Marking an episode as favorite from the card
- **WHEN** the episode's `favorite` is false and the user activates the card's favorite toggle
- **THEN** the backend stores favorite true, the card's star becomes filled, and a success notification is shown

#### Scenario: Unmarking a favorite from the card
- **WHEN** the episode's `favorite` is true and the user activates the card's favorite toggle
- **THEN** the backend stores favorite false, the card's star becomes hollow, and a notification is shown

#### Scenario: Toggle stays in sync with the shared state
- **WHEN** the same episode is rendered in more than one card (e.g. channel view and episodes view)
- **THEN** toggling in one place updates the star state everywhere the episode is rendered