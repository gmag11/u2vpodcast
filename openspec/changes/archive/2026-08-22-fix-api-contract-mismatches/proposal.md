## Why

Two API endpoints contradict their names/contracts:

1. `GET /api/1.0/channels/` is registered as `read_with_pagination` (`src/handlers/channels.rs:36`) but ignores pagination entirely and calls `Channel::read_all`, returning every channel in one response. The SPA grows unbounded payloads as channels accumulate, while the name promises `page`/`per_page` semantics that exist for users and episodes.
2. `CResponse::ko` (`src/models/response.rs:44`) always returns an HTTP **200** response with an embedded `status_code` field. Login failures (401), bad requests (400) and similar client errors are reported as successful HTTP exchanges; clients must parse the body to detect failure, and shared/global error handling that keys off HTTP status misbehaves.

## What Changes

- Implement real pagination for `GET /channels/` by honoring `page`/`per_page` (reusing the config `per_page` default), OR complete the existing `read_with_pagination` model method and bind it. The response stays in the `CResponse` envelope so the SPA surface is unchanged apart from the page subset.
- Make error responses carry the actual HTTP status code: `CResponse::ko` (and the `Error` path, which already returns the correct status) build responses with the proper status line. Response body shape remains identical.

## Capabilities

### New Capabilities

- `api-response-contract`: Defines that endpoints honor their declared pagination contract and that error responses report their real HTTP status.

### Modified Capabilities

(none)

## Impact

- `src/handlers/channels.rs` (pagination), possibly `src/models/channel.rs` (existing `read_with_pagination` is marked `#[allow(unused)]`), `src/models/response.rs` (`CResponse::ko`).
- Frontend: check any consumer of `CResponse::ko`/login-failure paths for assumptions about HTTP 200.
- No schema change.

## Non-Goals

- No change to the response *body* schema (`{status, status_code, message, user, data}` stays).
- No new pagination metadata (total counts) — the existing page/per_page contract elsewhere is preserved as-is.
- No change to the success-path response envelope.