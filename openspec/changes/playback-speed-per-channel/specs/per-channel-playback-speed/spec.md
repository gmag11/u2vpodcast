## ADDED Requirements

### Requirement: Playback speed is stored per channel

The system SHALL store a playback speed preference per channel, persisted server-side with the channel record (`channels.playback_speed`), defaulting to `1.0` for channels without an explicit value. Channel payloads returned by the channels API SHALL include the stored speed so clients can resolve it without extra requests. The stored value SHALL be a finite number within `0.5`–`3.0` expressed with at most two decimal places; values outside that range SHALL NOT be stored.

#### Scenario: Channel list includes the stored speed
- **WHEN** the client fetches the channels list and a channel already has a saved playback speed of `1.35`
- **THEN** the channel payload includes `playback_speed: 1.35`

#### Scenario: Channels without a saved speed default to 1.0
- **WHEN** the client fetches a channel (new or existing) that never had a speed saved
- **THEN** its payload reports `playback_speed: 1.0`

### Requirement: Episode payloads carry the channel's playback speed

Episode payloads returned by the episodes endpoints SHALL include the playback speed saved for the episode's channel (`playback_speed`), so the player can apply the correct rate at playback start without a separate lookup. An episode whose channel has no saved speed SHALL report `playback_speed: 1.0`.

#### Scenario: Episodes expose their channel's saved speed
- **WHEN** the client fetches episodes and one belongs to a channel with saved speed `1.7`
- **THEN** that episode's payload includes `playback_speed: 1.7`

#### Scenario: Episodes of an unconfigured channel default to 1.0
- **WHEN** the client fetches episodes of a channel that never had a speed saved
- **THEN** each episode's payload includes `playback_speed: 1.0`

### Requirement: Playback speed can be updated per channel

The system SHALL expose an endpoint to update a channel's playback speed (`PUT /api/1.0/channels/{slug}/playback_speed/`) accepting the new value in the request body. The server SHALL validate the value: a non-finite number, a value below `0.5`, or a value above `3.0` SHALL be rejected with a client error; a valid value SHALL be rounded to two decimal places, persisted, and the request answered with success (204 with no body). Updating the speed of an unknown channel SHALL answer with a not-found error.

#### Scenario: A valid speed overwrites the stored value
- **WHEN** the client sends `{ playback_speed: 1.35 }` for a channel whose stored speed is `1.0`
- **THEN** the stored speed becomes `1.35` and the request succeeds with 204

#### Scenario: An out-of-range speed is rejected
- **WHEN** the client sends `{ playback_speed: 4.0 }` (or `0.2`) for a channel
- **THEN** the server rejects the request with a client error and the stored speed remains unchanged

#### Scenario: Updating an unknown channel fails
- **WHEN** the client sends a speed update for a channel slug that does not exist
- **THEN** the server answers with a not-found error

### Requirement: Playback starts at the channel's saved speed

Whenever playback starts on an episode — a fresh `play`, resuming, queue navigation (next/previous/advance), or a restored queue — the shared player SHALL apply the playback speed saved for that episode's channel before or at the moment playback begins, using the speed carried by the episode payload when available. An episode whose channel has no saved speed SHALL play at `1.0`. The applied speed SHALL be reflected by the player's speed state and the persistent bar's speed control. Whenever the player switches to an episode of a different channel — because the previous episode ended and playback auto-advanced, or because the user skipped manually — the player SHALL load and apply the new channel's saved speed, and playback SHALL NOT inherit the previous channel's playback rate.

#### Scenario: Playing an episode applies its channel's speed
- **WHEN** the user plays an episode of a channel with saved speed `1.35`
- **THEN** playback starts at `1.35x` and the player reports speed `1.35`

#### Scenario: Playing an episode without a channel speed starts at 1x
- **WHEN** the user plays an episode whose channel has no saved speed
- **THEN** playback starts at `1.0x`

#### Scenario: Queue navigation re-applies the channel speed
- **WHEN** the user skips to the next episode in the queue and its channel has a different saved speed
- **THEN** the player switches to that channel's saved speed for the new episode

#### Scenario: Auto-advance into a different channel applies its saved speed
- **WHEN** the current episode ends and playback advances to the next queued episode whose channel has a different saved speed
- **THEN** the player applies the new channel's saved speed before the next episode starts playing

#### Scenario: Manual skip into a different channel applies its saved speed
- **WHEN** the user manually skips (next or previous) to an episode of a channel with a different saved speed
- **THEN** the player applies that channel's saved speed before the new episode plays

#### Scenario: Switching channels never carries over the previous rate
- **WHEN** an episode of channel A playing at `2x` is followed (by auto-advance or manual skip) by an episode of channel B with saved speed `1.35`
- **THEN** channel B's episode plays at `1.35x` and the previous `2x` rate is not carried over

#### Scenario: Skipping within the same channel keeps its saved speed
- **WHEN** the user advances or skips to an episode of the same channel
- **THEN** the speed remains the channel's saved value

### Requirement: Manual speed changes overwrite the saved channel speed

When the user changes the playback speed while an episode is loaded, the new value SHALL immediately apply to playback and SHALL become the saved speed for the episode's channel, overwriting any previously stored value. The overwrite SHALL be persisted server-side (fire-and-forget, mirroring playback-progress) and SHALL apply to all episodes of that channel from then on. Episodes of other channels SHALL keep their own saved speeds.

#### Scenario: Adjusting speed while playing saves the new value
- **WHEN** an episode of a channel with saved speed `1.0` is playing and the user sets the speed to `1.7`
- **THEN** playback continues at `1.7x` and the channel's stored speed becomes `1.7`

#### Scenario: The next episode of the same channel starts at the new speed
- **WHEN** the user has just saved speed `1.7` for a channel and then plays another episode of the same channel
- **THEN** that episode starts at `1.7x`

#### Scenario: Different channels keep independent speeds
- **WHEN** the user saved `1.35` for channel A and `2.0` for channel B and plays episodes of both
- **THEN** each episode starts at its own channel's speed and changing one does not change the other