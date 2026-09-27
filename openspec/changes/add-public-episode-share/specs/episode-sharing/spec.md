# Spec Delta

## Purpose

Lets an authenticated user share a single episode through a permanent, unguessable public link built from a signed hash of the episode's URI, that renders a browser player and serves only that episode's metadata and audio, without exposing the rest of the library or requiring the visitor to authenticate.

## ADDED Requirements

### Requirement: Authenticated users mint a share link for one episode

The system SHALL expose an authenticated `POST /api/1.0/episodes/{yt_id}/share/` endpoint, protected by the same session guard as the rest of the JSON API, that returns the permanent public share URL for the episode identified by `{yt_id}`. The response SHALL NOT include an expiry. When `{yt_id}` does not resolve to a stored episode, the endpoint SHALL respond `404 Not Found`. The returned URL SHALL point at the public share page and SHALL uniquely identify the episode.

#### Scenario: Sharing a stored episode
- **WHEN** a client with a valid session sends `POST /api/1.0/episodes/abc123/share/` for a stored episode
- **THEN** the system responds `200 OK` with a payload containing a public URL of the form `{url}/app/share/{token}` and no expiry field

#### Scenario: Unknown episode is rejected
- **WHEN** a client with a valid session sends `POST /api/1.0/episodes/does-not-exist/share/`
- **THEN** the system responds `404 Not Found` and mints no link

#### Scenario: Minting requires a session
- **WHEN** a client without a valid session sends `POST /api/1.0/episodes/abc123/share/`
- **THEN** the system responds `401 Unauthorized` with the standard JSON API error body

### Requirement: Share tokens are permanent signed hashes of the episode URI

A share token SHALL be `{yt_id}-{hash}`, where `{hash}` is the lowercase-hex HMAC-SHA256, under the deployment's effective share secret, of the episode's canonical URI (`webpage_url`) prefixed with a fixed domain-separation label. The token SHALL carry no expiry and SHALL be identical for the same episode and secret across requests. Verification SHALL load the episode from the token's `{yt_id}` and compare the recomputed hash in constant time; a token whose hash does not match the episode's URI, whose `{yt_id}` does not resolve, or that is malformed SHALL be treated as unavailable.

#### Scenario: Deterministic and permanent
- **WHEN** the same episode is shared twice under the same secret
- **THEN** both mints return the identical URL, and the link works regardless of how much time has passed

#### Scenario: Valid token is accepted
- **WHEN** a request presents a token whose recomputed hash matches the episode's URI and whose `yt_id` resolves
- **THEN** the request is served

#### Scenario: Tampered hash is rejected
- **WHEN** a token's hash does not match the episode's URI
- **THEN** the request is treated as unavailable and no episode data or audio is served

#### Scenario: Unknown episode is rejected
- **WHEN** a token's `yt_id` does not resolve to a stored episode
- **THEN** the request is treated as unavailable

#### Scenario: Malformed token is rejected
- **WHEN** a token has no usable `yt_id` or hash
- **THEN** the request is treated as unavailable

### Requirement: The share secret is configurable and falls back to the session secret

The system SHALL sign share hashes with a configured `share_secret`; when `share_secret` is absent or empty, it SHALL fall back to `secret_key`. Rotating the effective share secret SHALL invalidate every outstanding link. There SHALL be no per-link revocation and no time-based expiry: a link stops working only when its episode is deleted or the effective share secret is rotated.

#### Scenario: Rotating the share secret revokes links
- **WHEN** the effective share secret changes and the app restarts, then a client presents a link minted before the change
- **THEN** the recomputed hash no longer matches and the request is treated as unavailable

#### Scenario: Fallback to the session secret
- **WHEN** `share_secret` is absent or empty
- **THEN** links are signed with `secret_key` and verify against it

### Requirement: Public metadata endpoint for the shared episode

The system SHALL serve `GET /s/{token}/episode.json` without requiring a session cookie or Basic Auth, returning only the fields needed to render the share page for the token's episode: title, channel title, description, image, duration, publish date, and the tokenized audio URL. The response SHALL NOT include an expiry, user identity, playback progress, favorite state, chapters, SponsorBlock data, or any other episode. A malformed, forged, or unresolvable token SHALL respond `404 Not Found`.

#### Scenario: Valid token returns minimal metadata
- **WHEN** a client with no credentials requests `GET /s/{valid_token}/episode.json`
- **THEN** the system responds `200 OK` with that episode's title, channel title, description, image, duration, publish date, and audio URL, and no expiry, user, progress, favorite, or other-episode data

#### Scenario: Invalid token returns not found
- **WHEN** a client requests `GET /s/{token}/episode.json` with a malformed, forged, or unresolvable token
- **THEN** the system responds `404 Not Found`

#### Scenario: Endpoint is reachable even when feeds and media are protected
- **WHEN** `with_authentication` is `true` and a client with no cookie and no `Authorization` header requests the metadata for a valid token
- **THEN** the system serves the metadata instead of responding `401`

### Requirement: Public tokenized audio endpoint

The system SHALL serve `GET` and `HEAD /s/{token}/audio.mp3` without requiring a session cookie or Basic Auth, streaming the same representation the episode's stable media URL would serve: the active SponsorBlock-processed MP3 when SponsorBlock is enabled and a valid derivative exists, otherwise the original MP3. The response SHALL carry the audio content type and the same `Range` (including open-ended and suffix ranges), `ETag`, `Last-Modified`, and conditional-request (`304`) semantics as the protected media route. A malformed, forged, or unresolvable token SHALL respond `404 Not Found`, as SHALL a valid token whose media is missing on disk.

#### Scenario: Valid token streams the selected representation
- **WHEN** a client with no credentials requests `GET /s/{valid_token}/audio.mp3`
- **THEN** the system responds `200 OK` with the episode's selected audio bytes and `Content-Type: audio/mpeg`

#### Scenario: Range requests are honored
- **WHEN** a client sends a satisfiable `Range: bytes=start-end` header for `GET /s/{valid_token}/audio.mp3`
- **THEN** the system responds `206 Partial Content` with the corresponding byte range and `Content-Range`

#### Scenario: Invalid token serves no audio
- **WHEN** a client requests `GET /s/{token}/audio.mp3` with a malformed, forged, or unresolvable token
- **THEN** the system responds `404 Not Found` and streams no bytes

#### Scenario: Audio is served even when media is protected
- **WHEN** `with_authentication` is `true` and a client with no cookie and no `Authorization` header requests the audio for a valid token
- **THEN** the system streams the audio instead of responding `401`

### Requirement: Public share page plays the episode without a session

The SPA SHALL expose a public route at `/app/share/{token}` that renders the shared episode's cover, title, channel title, description, and an inline audio player whose source is the tokenized audio URL. The page SHALL load and play without an authenticated session, and an already-authenticated visitor SHALL NOT be redirected away from it. When the token is malformed, forged, or unresolvable, the page SHALL show an unavailable state instead of failing. The page SHALL NOT require or persist playback progress for the visitor.

#### Scenario: Anonymous visitor plays the shared episode
- **WHEN** a visitor with no session opens `/app/share/{valid_token}`
- **THEN** the page renders the episode metadata and a player that can play the tokenized audio

#### Scenario: Authenticated visitor is not redirected
- **WHEN** a visitor who already holds a valid session opens `/app/share/{valid_token}`
- **THEN** the share page renders instead of redirecting to the channels screen

#### Scenario: Unavailable link shows an unavailable state
- **WHEN** a visitor opens `/app/share/{malformed_or_forged_token}`
- **THEN** the page shows an unavailable state and offers no playback control

### Requirement: Authenticated share action on episode cards

An episode card SHALL offer a share action to authenticated users that requests a share link for that episode and copies it for the user, notifying on success and surfacing an error notification when minting fails.

#### Scenario: Sharing from an episode card
- **WHEN** an authenticated user triggers the share action on an episode card
- **THEN** the SPA requests a share link for that episode, copies the returned URL, and confirms success

#### Scenario: Minting failure is surfaced
- **WHEN** the share request fails
- **THEN** the SPA shows an error notification and copies nothing
