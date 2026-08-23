## Purpose

Extends the episode card with an add-to-playlist action defined by the `playlists` capability.

## ADDED Requirements

### Requirement: Add-to-playlist action on episode cards

Each episode card SHALL expose an action (dropdown or menu) that lets the user add the episode to an existing playlist or to a newly created playlist, with feedback on success.

#### Scenario: Adding an episode to an existing playlist
- **WHEN** the user opens the card's playlist action and picks an existing playlist
- **THEN** the episode is appended to that playlist and a success notification is shown

#### Scenario: Adding the same episode twice is prevented
- **WHEN** the episode is already in the chosen playlist
- **THEN** the action fails with a message and the playlist stays unchanged

#### Scenario: Creating a playlist while adding
- **WHEN** the user opens the card's playlist action, enters a new playlist name, and confirms
- **THEN** a new playlist is created and the episode is added to it