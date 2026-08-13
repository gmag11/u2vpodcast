## ADDED Requirements

### Requirement: A channel cover can be re-read on demand

The system SHALL expose `POST /api/1.0/channels/{id-or-slug}/image/` that resolves the channel by numeric id or slug, re-fetches its YouTube `og:image` cover URL (reusing the same metadata fetch used at channel creation, stripping query-string sizing params), persists the new URL to the channel's `image` column, and responds with the updated channel. The endpoint SHALL NOT download episodes or audio. The endpoint SHALL respond synchronously with the updated channel.

#### Scenario: Re-read the cover of a channel by slug
- **WHEN** an authenticated user sends `POST /api/1.0/channels/linux_y_tapas/image/`
- **THEN** the system re-fetches the YouTube cover URL for the channel with slug `linux_y_tapas`, stores it in the channel's `image` column, and responds `200 OK` with the updated channel including the new `image`

#### Scenario: Re-read the cover of a channel by id
- **WHEN** an authenticated user sends `POST /api/1.0/channels/3/image/`
- **THEN** the system resolves the channel with id `3` and re-reads its cover image URL the same way as by slug

#### Scenario: Re-reading the cover of an unknown channel fails
- **WHEN** an authenticated user sends `POST /api/1.0/channels/does_not_exist/image/` for a slug with no matching channel
- **THEN** the system responds with an error `CustomResponse` (channel not found) and updates nothing

#### Scenario: Cover re-fetch fails while fetching metadata
- **WHEN** the YouTube metadata fetch for a known channel fails (e.g., network error or blocked request)
- **THEN** the system responds with an error `CustomResponse`, the channel's stored `image` is left unchanged, and no episodes are affected

### Requirement: Channel card has a cover refresh button

The channel card on the dashboard SHALL display a small button for re-reading the channel's cover image. The button SHALL use a distinct image/cover icon (visually different from the episode-refresh control) and SHALL show a tooltip on hover explaining its purpose (e.g., "Reload cover"). Clicking it SHALL call `POST /api/1.0/channels/{slug}/image/` for the card's channel, show a loading state while the request is in flight (and prevent repeat clicks), update the card's image from the returned channel, and surface a success or error notification.

#### Scenario: User refreshes a channel cover from the card
- **WHEN** the user clicks the cover refresh button on a channel card
- **THEN** the SPA calls the image endpoint, shows a loading state on that button while waiting, updates the card's cover with the returned `image`, and shows a success notification

#### Scenario: Hovering the cover refresh button shows its purpose
- **WHEN** the user hovers over the cover refresh button on a channel card
- **THEN** a tooltip appears with text indicating the button reloads the cover image, distinguishing it from the episode refresh control

#### Scenario: Cover refresh request fails
- **WHEN** the image endpoint returns an error (e.g., channel not found or metadata fetch failure)
- **THEN** the SPA shows an error notification and the card keeps its previous image unchanged
