## Purpose

Defines the ability to trigger a channel's episode refresh on demand, outside the periodic worker cycle. A single channel can be refreshed out of cycle via an authenticated endpoint, and newly created channels start their update immediately after creation instead of waiting for the next worker run.

## Requirements

### Requirement: A channel can be refreshed on demand

The system SHALL expose `POST /api/1.0/channels/{slug}/update/` that resolves the channel by slug and triggers its episode refresh out of the periodic worker cycle. The endpoint SHALL respond immediately (before downloads complete) and run the refresh as an asynchronous background task.

#### Scenario: Refresh a channel by slug
- **WHEN** an authenticated user sends `POST /api/1.0/channels/linux_y_tapas/update/`
- **THEN** the system resolves the channel with slug `linux_y_tapas`, starts refreshing its episodes in the background, and responds `200 OK` with a `CustomResponse` immediately

#### Scenario: Refresh of an unknown channel fails
- **WHEN** an authenticated user sends `POST /api/1.0/channels/does_not_exist/update/` for a slug with no matching channel
- **THEN** the system responds with an error `CustomResponse` (channel not found) and does not start any download

### Requirement: Refreshing a channel downloads its new episodes

A triggered refresh SHALL run the same logic as the periodic worker for that single channel: create the channel's audios directory, fetch new videos via yt-dlp, download their audio, store episode rows, and clean episodes beyond the channel's max. The refresh SHALL NOT affect other channels.

#### Scenario: Refresh downloads new episodes
- **WHEN** a channel has episodes published after its last stored date and the user triggers an update
- **THEN** the background refresh downloads those episodes and stores them, and they appear in `GET /api/1.0/channels/{id}/episodes/`

### Requirement: Creating a channel starts its update immediately

When a new channel is created via `POST /api/1.0/channels/`, the system SHALL trigger the channel's refresh automatically right after the row is created, so episodes begin downloading without waiting for the periodic cycle.

#### Scenario: New channel refreshes right after creation
- **WHEN** an authenticated user creates a channel
- **THEN** the channel is stored and its episode refresh starts immediately in the background (it does not wait for the worker's sleep cycle)

### Requirement: Episodes screen has a refresh control

The episodes page (`/app/{channelId}`) SHALL display a "Refresh" button. Clicking it SHALL call `POST /api/1.0/channels/{slug}/update/` for the channel being viewed, show a loading state while the request is in flight, and surface a success/error notification.

#### Scenario: User refreshes the current channel
- **WHEN** the user is viewing a channel's episodes and clicks Refresh
- **THEN** the SPA calls the update endpoint, shows a loading indicator, and displays a success notification on completion

#### Scenario: Refresh request fails
- **WHEN** the update endpoint returns an error (e.g., channel not found or network failure)
- **THEN** the SPA shows an error notification and keeps the episodes list unchanged
