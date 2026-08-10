## Why

The `<audio>` element in the SPA loads media files from `/media/{slug}/{yt_id}.mp3`, which is protected exclusively by HTTP Basic Auth. Even though the user is already logged in via the session cookie, the browser has no cached Basic Auth credential on a new device or after clearing saved passwords, so it prompts for credentials. The same applies to feed URLs accessed from a browser with a valid session. Making both routes accept the session cookie in addition to Basic Auth eliminates the spurious prompt for logged-in SPA users without breaking podcast clients (which keep using Basic Auth).

## What Changes

- Create a new `SessionOrBasicAuth` middleware in `src/utils/middleware.rs` that passes the request through if **either** a valid session cookie (resolved via the existing `from_session`) **or** a valid Basic Auth header is present.
- Replace `BasicAuthGuard` with `SessionOrBasicAuth` on the RSS feed routes (`/channels/{slug}/feed.xml` and `/{slug}/feed.xml`) and on the `/media/**` scope in `src/handlers/mod.rs`.
- The `with_authentication` flag continues to bypass the guard entirely when `false`, same as today.

## Capabilities

### Modified Capabilities
- `route-protection`: the RSS feed and static media requirements change from "requires Basic Auth only" to "requires a valid session cookie OR valid Basic Auth credentials". The JSON API session guard, anonymous endpoints, credential store, and the `with_authentication` toggle are unchanged.

## Impact

- **Code**: `src/utils/middleware.rs` (new `SessionOrBasicAuth` middleware), `src/handlers/feed.rs` (replace `BasicAuthGuard` with `SessionOrBasicAuth`), `src/handlers/mod.rs` (replace `BasicAuthGuard` on `/media` with `SessionOrBasicAuth`).
- **Dependencies**: none.
- **APIs**: no new endpoints. Existing endpoints accept one additional auth method (session cookie); their response shape and status codes are identical.
- **DB**: none.
- **Frontend**: none — the SPA already sends the session cookie on same-origin requests to `/media` and the feed URLs.
