## Why

The web player currently relies on browser-default media behavior, so operating-system controls may pause or resume audio but cannot reliably navigate the episode queue, expose useful episode metadata, or remain synchronized with the application. Explicit Media Session integration is needed to provide predictable controls from desktop media keys, mobile lock screens, notification controls, and connected headsets while preserving playback on browsers that do not support the API.

## What Changes

- Register supported operating-system media actions for play, pause, next track, previous track, relative seek, and absolute seek.
- Route every system action through the existing player store so queue navigation, resume behavior, SponsorBlock handling, progress persistence, and UI state continue to use one playback path.
- Publish the current episode title, channel, and artwork to the operating system.
- Keep the exposed playback state and position synchronized with the underlying audio element when the browser supports those Media Session features.
- Feature-detect all Media Session functionality and preserve the current player behavior when it is unavailable or individual actions are unsupported.
- Add automated contract tests and a manual mobile/desktop compatibility matrix for system media controls.

## Capabilities

### New Capabilities

- `system-media-controls`: Defines operating-system media actions, episode metadata, playback/position synchronization, and graceful degradation across supported mobile and desktop browsers.

### Modified Capabilities

None.

## Impact

- Frontend player state and audio lifecycle in `frontend/src/stores/player.ts`.
- Player store tests and Media Session browser-API test doubles.
- Manual verification on representative desktop and mobile browser/OS combinations.
- No backend API, database schema, media URL, or third-party runtime dependency changes are expected.
