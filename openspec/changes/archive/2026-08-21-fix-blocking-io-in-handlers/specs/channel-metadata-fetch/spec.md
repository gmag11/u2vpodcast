## ADDED Requirements

### Requirement: Channel metadata fetch does not block the async runtime

Fetching channel metadata (title, description, image) from the upstream host SHALL be performed off the tokio worker threads, either through `spawn_blocking` for a synchronous HTTP client or with an asynchronous HTTP client. While one or more metadata requests run, other API requests (login, session, channels, episodes) SHALL continue to be served without waiting for those fetches.

#### Scenario: Slow metadata fetch does not stall other requests
- **WHEN** a `POST /api/1.0/channels/` metadata request hangs upstream while others requests are in flight
- **THEN** other API endpoints (e.g. `GET /api/1.0/channels/`, `POST /api/1.0/login/`) still respond promptly

#### Scenario: Two concurrent channel creations do not freeze the API
- **WHEN** two channel-creation requests each perform their metadata fetch simultaneously
- **THEN** both complete and the rest of the API remains responsive

### Requirement: Metadata fetch is time-bounded

The metadata HTTP request SHALL have an explicit timeout; a fetch that exceeds it SHALL fail with the request's normal error path (channel creation fails cleanly, image refresh returns an error) within bounded time instead of waiting indefinitely.

#### Scenario: Hung upstream returns error after timeout
- **WHEN** the upstream does not respond within the configured timeout
- **THEN** the metadata fetch errors out at the timeout, the failing request is rejected with the existing error handling, and no request thread is left blocked

#### Scenario: Healthy upstream fetch succeeds unchanged
- **WHEN** the upstream responds quickly
- **THEN** metadata is extracted exactly as before and the channel/image update proceeds