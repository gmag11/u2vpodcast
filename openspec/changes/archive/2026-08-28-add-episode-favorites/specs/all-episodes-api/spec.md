# all-episodes-api

## Purpose

Backend API capability that exposes every episode from every channel in a single list, newest first, with each episode annotated by its owning channel's slug and title. Enables cross-channel views like the history screen without a second lookup.

## ADDED Requirements

### Requirement: All-episodes payloads include the favorite flag

Each episode returned by the all-episodes endpoint SHALL include a boolean `favorite` field reflecting the stored favorite flag, so cross-channel views can render the favorite state per card without extra requests.

#### Scenario: Favorite flag present on each episode
- **WHEN** an authenticated user requests `GET /api/1.0/episodes/`
- **THEN** every returned episode object includes `favorite` with the value stored for that episode