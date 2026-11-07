# Spec Delta

## ADDED Requirements

### Requirement: Public share routes are exempt from the session and Basic-Auth guard

The system SHALL serve `GET /s/{token}/episode.json` and `GET`/`HEAD /s/{token}/audio.mp3` without requiring a session cookie or an `Authorization: Basic` header, even when `with_authentication` is `true`, provided the token is valid. These routes SHALL be authorized solely by the signed token and SHALL NOT become reachable through any other value. A malformed, forged, or unresolvable token SHALL respond `404 Not Found` rather than `401`. The authenticated endpoint that mints tokens (`POST /api/1.0/episodes/{yt_id}/share/`) SHALL remain protected by the JSON API session guard.

#### Scenario: Valid token without credentials is served while authentication is on
- **WHEN** `with_authentication` is `true` and a client with no cookie and no `Authorization` header requests the metadata or audio for a valid token
- **THEN** the request is served, without a `WWW-Authenticate` challenge

#### Scenario: Missing or invalid token is not an authentication challenge
- **WHEN** a client with no credentials requests a share route with a missing, malformed, forged, or unresolvable token
- **THEN** the system responds `404 Not Found` and does not respond `401`

#### Scenario: Public share routes are token-gated regardless of the flag
- **WHEN** `with_authentication` is `false` and a client requests a share route with an invalid token
- **THEN** the system still responds `404 Not Found`

#### Scenario: Minting tokens still requires a session
- **WHEN** a client without a valid session sends `POST /api/1.0/episodes/{yt_id}/share/`
- **THEN** the system responds `401 Unauthorized` with the standard JSON API error body
