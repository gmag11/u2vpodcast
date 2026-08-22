# youtube-throttling

## Purpose

Defines the single-connection policy for all outbound YouTube traffic: metadata fetches, cover image fetches, and every yt-dlp execution are serialized through one global slot with a post-connection cooldown, so a burst of triggers (forced refreshes, concurrent creates, scheduled cycles) can never fire parallel connections that YouTube may treat as bot behavior.

## Requirements

### Requirement: At most one YouTube connection at any time

All YouTube-bound operations SHALL be serialized through a single global throttle: reading channel metadata, reading cover images, and executing yt-dlp runs. While one such operation has an outbound connection in progress, any other YouTube-bound operation SHALL wait for the slot rather than starting a parallel connection. This SHALL hold regardless of the trigger: scheduled worker cycles, a forced refresh of several channels, or concurrent manual API requests.

#### Scenario: Concurrent channel creates serialize metadata fetches
- **WHEN** several channel-create requests arrive at the same time
- **THEN** their metadata fetches run one after another, never overlapping

#### Scenario: Forced refresh of many channels serializes yt-dlp runs
- **WHEN** an operator forces a refresh of multiple channels at once
- **THEN** the yt-dlp executions run strictly one at a time

#### Scenario: Metadata fetch and yt-dlp never overlap
- **WHEN** a channel-create metadata fetch is in flight while the worker is about to run yt-dlp
- **THEN** the yt-dlp run waits until the metadata connection has finished and its cooldown elapsed

#### Scenario: Cover image refresh respects the slot
- **WHEN** a cover image refresh is requested while another YouTube connection is active or cooling down
- **THEN** the image fetch waits for the slot instead of opening a parallel connection

### Requirement: Cooldown after every connection

After each YouTube connection ends, whether it succeeded or errored, the throttle SHALL hold the slot for a cooldown period before allowing the next connection to start. The cooldown SHALL be configurable (in seconds) with a sensible default applied when the option is absent, and SHALL be identical for every operation type.

#### Scenario: Successful connection is followed by the configured pause
- **WHEN** a metadata fetch succeeds
- **THEN** the next YouTube connection starts only after the cooldown has elapsed

#### Scenario: Failed connection also enforces the cooldown
- **WHEN** a YouTube connection fails (e.g. network error or HTTP rejection)
- **THEN** the next connection still waits for the cooldown, so a burst of errors cannot turn into rapid-fire retries

#### Scenario: Cooldown configured via option
- **WHEN** the configured value is e.g. 3 seconds
- **THEN** consecutive YouTube connections are at least 3 seconds apart; when the option is absent the default is used

### Requirement: The throttle cannot be bypassed

There SHALL be no code path that performs a YouTube connection outside the throttle, including the periodic yt-dlp update check and any fallback/error branches. Waiters SHALL resume in bounded time when the slot holder fails or panics, so a stuck connection cannot deadlock the throttle.

#### Scenario: yt-dlp update check is throttled too
- **WHEN** the worker performs the periodic yt-dlp update check
- **THEN** it acquires the throttle slot like any other YouTube operation

#### Scenario: Slot holder failure does not deadlock waiters
- **WHEN** the connection currently holding the slot panics or fails
- **THEN** the slot is released and the next waiter proceeds without waiting indefinitely (no deadlock)