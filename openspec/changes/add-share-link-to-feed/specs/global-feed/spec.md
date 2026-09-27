# Spec Delta

## ADDED Requirements

### Requirement: Global feed items carry a deterministic public share link

Each `<item>` in the global feed SHALL set `<link>` to the episode's public share page URL `{url}/app/share/{token}`, where `{token}` is the episode's permanent, deterministic share token (`{yt_id}-{hash(share_secret, "v1|{webpage_url}")}`). This SHALL be in addition to the existing channel-distinct `<enclosure>` and prefixed title, which SHALL remain unchanged. Because the token is deterministic, the same `<link>` appears on every build of the global feed.

#### Scenario: Aggregated item exposes its public share page

- **WHEN** a client with valid credentials requests `/feed.xml`
- **THEN** every `<item>` contains a `<link>` beginning with `{url}/app/share/`, alongside its existing `<enclosure>` and channel-prefixed title

#### Scenario: The link token resolves to the aggregated item's episode

- **WHEN** the token carried by a global-feed item's `<link>` is submitted to `GET /s/{token}/episode.json` without credentials
- **THEN** the response resolves to the same `yt_id` as that item's `<guid>`

#### Scenario: Aggregated enclosure and title are unchanged

- **WHEN** the global feed is built for an episode of channel `confesiones_de_gasolinera`
- **THEN** its `<enclosure>` remains the owning channel's selected media URL and its title remains prefixed with the channel title, exactly as before the link was added
