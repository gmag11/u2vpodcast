# channel-image-cache

## Purpose

Defines the local, authenticated cache for channel cover images: each channel's cover is stored as a local file served by the app, refreshed on every channel update (create, manual refresh, and worker sync) with a size-based skip that avoids redundant downloads, and persisted inside an existing Docker volume so container recreation does not lose it.

## Requirements

### Requirement: Channel cover images are cached locally

Each channel's cover image SHALL be stored as a local file served by the application, and the `image` field returned by the channel API SHALL reference that local URL instead of the remote YouTube URL. Rendering the SPA SHALL NOT cause the browser to open connections to YouTube for cover images.

#### Scenario: Page load serves cached images locally
- **WHEN** the SPA renders the channel list
- **THEN** every `channel.image` value points to a local URL served by the app with no request reaching YouTube

#### Scenario: Cached image URL is stable per channel
- **WHEN** the API returns a channel that has a cached image
- **THEN** the image URL is derived deterministically from the channel (e.g. its slug) and does not change between responses unless the cache is refreshed

### Requirement: Cache is refreshed on every channel update

The cached image SHALL be re-downloaded and replaced whenever the channel's metadata is refreshed: at channel creation, on an explicit cover-image refresh, and during each scheduled/forced channel update (sync). A failed download SHALL keep the previous cached file and SHALL NOT blank the channel's image. Deleting a channel SHALL also delete its cached image file.

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

#### Scenario: Channel deletion removes the cached image
- **WHEN** a channel is deleted
- **THEN** its cached image file is removed together with the channel's audio directory

### Requirement: Unchanged images are not re-downloaded

Before downloading, the app SHALL probe the remote image URL with an HTTP `HEAD` request and compare the reported `Content-Length` with the size of the existing cached file. When a cached file exists and its size matches the reported length, the download SHALL be skipped. If the probe fails or reports no size, the app SHALL fall back to a full bounded download.

#### Scenario: Same-sized image skips the download
- **WHEN** a channel update finds an existing cached file whose size equals the `Content-Length` from the `HEAD` probe
- **THEN** the image is not downloaded again and the cached file is kept as-is

#### Scenario: Changed size triggers a fresh download
- **WHEN** the `HEAD` probe reports a `Content-Length` different from the current cached file size (or no cached file exists)
- **THEN** the image is downloaded and atomically replaces the cached file

#### Scenario: Probe failure falls back to a bounded download
- **WHEN** the `HEAD` request errors, times out, or omits `Content-Length`
- **THEN** the app performs a bounded full download instead of failing the refresh

### Requirement: The cache is stored inside an existing Docker volume

In the container, the cache directory SHALL live under an already-mounted volume so no new Docker volume or compose change is required. The preferred location is `/app/db/images` (the `db` volume). It SHALL NOT be placed in the audio volume where a channel slug such as `images` could collide with cache files.

#### Scenario: Cache survives container recreation
- **WHEN** the container is recreated and the volumes are kept
- **THEN** the cached images are still present under `/app/db/images`

#### Scenario: No new volume is added
- **WHEN** the deployment is updated
- **THEN** `docker-compose.yml` requires no new volume entries for the image cache

### Requirement: Cached images are served with authentication

The image cache SHALL be served through an authenticated route (the same `SessionOrBasicAuth` as `/media`); unauthenticated requests SHALL be rejected. Only files belonging to the cache directory SHALL be exposed.

#### Scenario: Authenticated session loads a cached image
- **WHEN** a logged-in browser requests a cached image URL from the same origin
- **THEN** the image bytes are returned (the session cookie authenticates the request)

#### Scenario: Unauthenticated image request is rejected
- **WHEN** a client without a valid session or basic credentials requests a cached image
- **THEN** the request is refused (e.g. `401`) and no image bytes are returned

#### Scenario: Cache route cannot serve arbitrary paths
- **WHEN** a client requests a path outside the cache directory
- **THEN** the request is not served from the cache route (no directory traversal outside the cache)

### Requirement: Missing cache degrades gracefully

A channel with no cached image (never fetched or fetch failed) SHALL keep the previous fallback behavior (empty image until the first successful fetch).

#### Scenario: Channel without cache shows no image until first fetch
- **WHEN** a channel has never had a successful image fetch
- **THEN** its `image` remains empty (or the previous fallback) and no broken local URL is returned