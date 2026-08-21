## ADDED Requirements

### Requirement: Channel cover images are cached locally

Each channel's cover image SHALL be stored as a local file served by the application, and the `image` field returned by the channel API SHALL reference that local URL instead of the remote YouTube URL. Rendering the SPA SHALL NOT cause the browser to open connections to YouTube for cover images.

#### Scenario: Page load serves cached images locally
- **WHEN** the SPA renders the channel list
- **THEN** every `channel.image` value points to a local URL served by the app with no request reaching YouTube

#### Scenario: Cached image URL is stable per channel
- **WHEN** the API returns a channel that has a cached image
- **THEN** the image URL is derived deterministically from the channel (e.g. its slug) and does not change between responses unless the cache is refreshed

### Requirement: Cache is refreshed on every channel update

The cached image SHALL be re-downloaded and replaced whenever the channel's metadata is refreshed: at channel creation, on an explicit cover-image refresh, and during each scheduled/forced channel update (sync). A failed download SHALL keep the previous cached file and SHALL NOT blank the channel's image.

#### Scenario: New channel populates the cache
- **WHEN** a channel is created with a successful metadata fetch
- **THEN** its cover image is downloaded into the cache and `channel.image` points to the local copy

#### Scenario: Manual image refresh replaces the cached file
- **WHEN** the user triggers the existing cover-image refresh for a channel
- **THEN** the cached file is replaced with the newly downloaded bytes (on success)

#### Scenario: Sync refreshes the cache
- **WHEN** the worker updates a channel on a scheduled or forced cycle
- **THEN** the channel's cached image is refreshed as part of that update

#### Scenario: Failed download keeps the previous image
- **WHEN** an image refresh or sync fetch fails
- **THEN** the existing cached file is kept and the channel's image remains the previous local URL

### Requirement: Missing cache degrades gracefully and is public

A channel with no cached image (never fetched or fetch failed) SHALL keep the previous fallback behavior (empty image until the first successful fetch). Cached image files SHALL be served without authentication, like the SPA static assets, and only files belonging to the cache directory SHALL be exposed.

#### Scenario: Channel without cache shows no image until first fetch
- **WHEN** a channel has never had a successful image fetch
- **THEN** its `image` remains empty (or the previous fallback) and no broken local URL is returned

#### Scenario: Cache route cannot serve arbitrary paths
- **WHEN** a client requests a path outside the cache directory
- **THEN** the request is not served from the cache route (no directory traversal outside the cache)