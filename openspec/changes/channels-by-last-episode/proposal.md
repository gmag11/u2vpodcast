## Why

The channel list should be sortable (alphabetical, by id, by last update), but for now only ordering by last update is needed. To keep that flexible, sorting SHALL live in the frontend, not in the API. The frontend therefore needs each channel's last episode publication date, which the API does not currently expose.

## What Changes

- The backend SHALL include a `last_date` field on every channel returned by `GET /api/1.0/channels/`, holding `MAX(episodes.published_at)` for that channel (`null` when the channel has no episodes).
- The frontend SHALL sort the channel list by `last_date` descending (newest first), placing channels without a `last_date` last.
- Sorting is implemented in a single frontend location so future sort keys (alphabetical, id) can be added without touching the API.

## Capabilities

### New Capabilities
- `channels-list-ordering`: default frontend ordering of the channel list by most recent episode, newest first, backed by a `last_date` field exposed on channel API responses.

### Modified Capabilities
<!-- No existing spec requirement covers channel list ordering or the channel payload shape, so no delta spec is needed -->

## Impact

- `src/models/channel.rs`: `Channel` struct gains `last_date: Option<DateTime<Utc>>`; `from_row` reads it tolerantly (missing column → `None`); `read_all` selects `c.*, e.last_date` via a `LEFT JOIN` over `MAX(episodes.published_at)`.
- `frontend/src/types.ts`: `Channel` gains `last_date: string | null`.
- `frontend/src/views/ChannelsView.vue`: `sortedChannels` computed feeds the filtered/paginated list; comparator sorts `last_date` desc, nulls last.
- No schema migration, no new routes, no change to create/update/read single-channel payloads beyond the new nullable field.
