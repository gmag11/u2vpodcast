## Purpose

Makes the channel name on the compact `EpisodeCard` variant (used in the history list) a link to that channel's episodes view, while keeping the link independent of playback controls.

## Requirements

### Requirement: Channel name on compact cards is a link to the channel's episodes

The compact `EpisodeCard` variant (used in the history list) SHALL render the channel name as a link that navigates to the episodes view of that channel. The link SHALL use the episode's `channel_id` and the existing named `episodes` route. The default (episode list) card variant SHALL remain unchanged.

#### Scenario: Clicking the channel name opens the channel's episode list
- **WHEN** the user clicks the channel name on a compact episode card in the history list
- **THEN** the app navigates to the episodes view for that channel (`/app/:channelId`), listing that channel's episodes

#### Scenario: Link target uses the episode's channel id
- **WHEN** a compact card renders its channel name link
- **THEN** the link targets the episodes route with `params.channelId` equal to the episode's `channel_id`

#### Scenario: Default card variant is unchanged
- **WHEN** an episode card in the channel episode list (non-compact) is rendered
- **THEN** the card shows no channel name link, matching current behavior

### Requirement: Link does not trigger playback

The channel name link SHALL be an independent element from the card's play/stop controls. Clicking it SHALL navigate to the channel's episode list without starting, pausing, or stopping audio playback.

#### Scenario: Clicking the channel name keeps audio state
- **WHEN** the user clicks the channel name link while an episode is playing
- **THEN** the app navigates to the channel's episode list and the current playback state is preserved
