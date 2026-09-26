# Spec Delta

## Purpose

Lets an authenticated user share a single episode through an unguessable, expiring public link that renders a browser player and serves only that episode's metadata and audio, without exposing the rest of the library or requiring the visitor to authenticate.

## ADDED Requirements

### Requirement: Authenticated users mint a share link for one episode

The system SHALL expose an authenticated `POST /api/1.0/episodes/{yt_id}/share/` endpoint, protected by the same session guard as the rest of the JSON API, that returns the public share URL for the episode identified by `{yt_id}` and the instant at which the link expires. When `{yt_id}` does not resolve to a stored episode, the endpoint SHALL respond `404 Not Found`. The returned URL SHALL point at the public share page and SHALL uniquely identify the episode and its expiry through the token it carries.

#### Scenario: Sharing a stored episode
- **WHEN** a client with a valid session sends `POST /api/1.0/episodes/abc123/share/` for a stored episode
- **THEN** the system responds `200 OK` with a payload containing a public URL of the form `{url}/app/share/{token}` and an `expires_at` about 30 days in the future

#### Scenario: Unknown episode is rejected
- **WHEN** a client with a valid session sends `POST /api/1.0/episodes/does-not-exist/share/`
- **THEN** the system responds `404 Not Found` and mints no link

#### Scenario: Minting requires a session
- **WHEN** a client without a valid session sends `POST /api/1.0/episodes/abc123/share/`
- **THEN** the system responds `401 Unauthorized` with the standard JSON API error body

### Requirement: Share tokens are signed, episode-bound, and expiring

A share token SHALL be a signed, non-guessable value that binds exactly one episode (`yt_id`) and an expiry instant. The signature SHALL be produced with a key derived from the deployment's existing `secret_key` using domain separation, so no separate share secret is configured and the token cannot be forged without that key. A request whose token is malformed, has an invalid signature, does not match the requested episode, or has expired SHALL be treated as unavailable. The token lifetime SHALL default to 30 days and SHALL be configurable.

#### Scenario: Valid token is accepted
- **WHEN** a request presents a token whose signature verifies, whose episode matches the requested resource, and whose expiry is in the future
- **THEN** the request is served

#### Scenario: Tampered token is rejected
- **WHEN** a request presents a token whose signature does not verify, or whose signature is valid but does not match the episode in its payload
- **THEN** the request is treated as unavailable and no episode data or audio is served

#### Scenario: Expired token is rejected
- **WHEN** a token's expiry instant has passed
- **THEN** the request is treated as unavailable even though the signature is valid

#### Scenario: Configurable lifetime
- **WHEN** the deployment sets a non-default share lifetime
- **THEN** newly minted links expire after that lifetime

### Requirement: Rotating the signing secret revokes outstanding share links

Because share tokens carry no server-side state, rotating `secret_key` and restarting SHALL invalidate every previously issued share link. The system SHALL NOT provide per-link revocation.

#### Scenario: Secret rotation invalidates links
- **WHEN** the deployment changes `secret_key` and restarts, then a client presents a link minted before the change
- **THEN** the token's signature no longer verifies and the request is treated as unavailable

### Requirement: Public metadata endpoint for the shared episode

The system SHALL serve `GET /s/{token}/episode.json` without requiring a session cookie or Basic Auth, returning only the fields needed to render the share page for the token's episode: title, channel title, description, image, duration, publish date, expiry, and the tokenized audio URL. The response SHALL NOT include user identity, playback progress, favorite state, chapters, SponsorBlock data, or any other episode. A malformed, forged, or expired token SHALL respond `404 Not Found`.

#### Scenario: Valid token returns minimal metadata
- **WHEN** a client with no credentials requests `GET /s/{valid_token}/episode.json`
- **THEN** the system responds `200 OK` with that episode's title, channel title, description, image, duration, publish date, expiry, and audio URL, and no user, progress, favorite, or other-episode data

#### Scenario: Invalid or expired token returns not found
- **WHEN** a client requests `GET /s/{token}/episode.json` with a malformed, forged, or expired token
- **THEN** the system responds `404 Not Found`

#### Scenario: Endpoint is reachable even when feeds and media are protected
- **WHEN** `with_authentication` is `true` and a client with no cookie and no `Authorization` header requests the metadata for a valid token
- **THEN** the system serves the metadata instead of responding `401`

### Requirement: Public tokenized audio endpoint

The system SHALL serve `GET` and `HEAD /s/{token}/audio.mp3` without requiring a session cookie or Basic Auth, streaming the same representation the episode's stable media URL would serve: the active SponsorBlock-processed MP3 when SponsorBlock is enabled and a valid derivative exists, otherwise the original MP3. The response SHALL carry the audio content type and the same `Range` (including open-ended and suffix ranges), `ETag`, `Last-Modified`, and conditional-request (`304`) semantics as the protected media route. A malformed, forged, or expired token SHALL respond `404 Not Found`, as SHALL a valid token whose media is missing on disk.

#### Scenario: Valid token streams the selected representation
- **WHEN** a client with no credentials requests `GET /s/{valid_token}/audio.mp3`
- **THEN** the system responds `200 OK` with the episode's selected audio bytes and `Content-Type: audio/mpeg`

#### Scenario: Range requests are honored
- **WHEN** a client sends a satisfiable `Range: bytes=start-end` header for `GET /s/{valid_token}/audio.mp3`
- **THEN** the system responds `206 Partial Content` with the corresponding byte range and `Content-Range`

#### Scenario: Expired token serves no audio
- **WHEN** a client requests `GET /s/{expired_token}/audio.mp3`
- **THEN** the system responds `404 Not Found` and streams no bytes

#### Scenario: Audio is served even when media is protected
- **WHEN** `with_authentication` is `true` and a client with no cookie and no `Authorization` header requests the audio for a valid token
- **THEN** the system streams the audio instead of responding `401`

### Requirement: Public share page plays the episode without a session

The SPA SHALL expose a public route at `/app/share/{token}` that renders the shared episode's cover, title, channel title, description, and an inline audio player whose source is the tokenized audio URL. The page SHALL load and play without an authenticated session, and an already-authenticated visitor SHALL NOT be redirected away from it. When the token is malformed, forged, or expired, the page SHALL show an unavailable state instead of failing. The page SHALL NOT require or persist playback progress for the visitor.

#### Scenario: Anonymous visitor plays the shared episode
- **WHEN** a visitor with no session opens `/app/share/{valid_token}`
- **THEN** the page renders the episode metadata and a player that can play the tokenized audio

#### Scenario: Authenticated visitor is not redirected
- **WHEN** a visitor who already holds a valid session opens `/app/share/{valid_token}`
- **THEN** the share page renders instead of redirecting to the channels screen

#### Scenario: Unavailable link shows an unavailable state
- **WHEN** a visitor opens `/app/share/{expired_token}` or a malformed token
- **THEN** the page shows an unavailable state and offers no playback control

### Requirement: Authenticated share action on episode cards

An episode card SHALL offer a share action to authenticated users that requests a share link for that episode and copies it for the user, notifying on success and surfacing an error notification when minting fails.

#### Scenario: Sharing from an episode card
- **WHEN** an authenticated user triggers the share action on an episode card
- **THEN** the SPA requests a share link for that episode, copies the returned URL, and confirms success

#### Scenario: Minting failure is surfaced
- **WHEN** the share request fails
- **THEN** the SPA shows an error notification and copies nothing
