# Spec Delta

## ADDED Requirements

### Requirement: Each feed item links to a deterministic public share page

For each `<item>` in a channel's RSS feed, the system SHALL set `<link>` to the episode's public share page URL `{url}/app/share/{token}`, where `{token}` is the episode's permanent, deterministic share token (`{yt_id}-{hash(share_secret, "v1|{webpage_url}")}`). Because the token is deterministic, the same channel's feed SHALL contain the same `<link>` on every build. The enclosure, `<guid>`, `<pubDate>`, description, ordering, and the feed's credential protection SHALL remain unchanged.

#### Scenario: Item link is the public share page

- **WHEN** a client with valid credentials requests `/channels/{slug}/feed.xml` for a channel with episodes
- **THEN** every `<item>` contains a `<link>` beginning with `{url}/app/share/`, while its `<enclosure>`, `<guid>`, and `<pubDate>` are unchanged from before this change

#### Scenario: The link is stable across builds

- **WHEN** the same channel's feed is requested twice
- **THEN** each episode's `<link>` is identical in both responses

#### Scenario: The link token resolves to the item's episode

- **WHEN** the token carried by an item's `<link>` is submitted to `GET /s/{token}/episode.json` without credentials
- **THEN** the response resolves to the same `yt_id` as that item's `<guid>`

#### Scenario: Feed protection is unchanged

- **WHEN** a client without credentials and without an `Authorization` header requests the feed
- **THEN** the system still responds `401 Unauthorized`, and adding `<link>` does not make the feed public
