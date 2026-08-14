## Why

Channel cards show a cover, title, and description, but nothing about recency. Users cannot tell at a glance whether a channel has recent content or has gone stale. Showing how old the last episode is solves that.

## What Changes

- Add a badge in the top-right corner of each channel card showing the age of the channel's last episode using the `last_date` field already exposed by the API.
- Format: truncated (no fractions) — days `Nd`, weeks `Nw`, months `Nm`, years `Ny` (e.g. `2d`, `3w`, `6m`, `3y`). A week and a half shows `1w`.
- Cards whose channel has no `last_date` (no episodes yet) SHALL show no badge.

## Capabilities

### New Capabilities
- `channel-card-age-badge`: relative-age indicator for the last episode on channel cards.

### Modified Capabilities
<!-- No existing spec requirement covers channel card content beyond layout, so no delta spec is needed -->

## Impact

- `frontend/src/components/ChannelCard.vue`: add a relative-positioned container and an absolutely-positioned badge in the top-right corner, plus a small age-formatting helper.
- `Channel.last_date` already exists (string | null) — no API/backend changes.
