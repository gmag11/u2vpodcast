## ADDED Requirements

### Requirement: Episode playback is app-wide, not card-scoped

Episode playback SHALL be managed by a global audio player shared across the app. The `EpisodeCard` SHALL NOT own a private `<audio>` element or private playback state; instead it SHALL bind its controls (play/pause, seek, volume, speed) to the global player store. The app shell SHALL render the persistent bottom player. This replaces the previous per-card player behavior while preserving the same media URL pattern (`/media/{slug}/{yt_id}.mp3`).

#### Scenario: Card playback uses the global player
- **WHEN** the user presses play in an episode card
- **THEN** the card drives the global player store, the shared `<audio>` element plays the episode, and the persistent bottom bar appears

#### Scenario: No per-card audio duplication
- **WHEN** an episode card renders
- **THEN** it contains no private `<audio>` element and no independent playback state; its player UI is bound to the shared store
