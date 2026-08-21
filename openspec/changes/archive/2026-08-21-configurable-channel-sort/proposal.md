## Why

The channel list currently orders by last episode date descending only, with no
way for the user to change it. Users want to sort by title (alphabetical) and by
id, and to control the sort direction (ascending/descending) directly from the
channels view. The existing `channels-list-ordering` spec already mandates that
ordering lives in the frontend so extra sort keys are additive without API
changes; this change delivers those keys and the controls to configure them.

## What Changes

- The channels view SHALL expose sort controls: a sort key selector (last
  episode date — the default —, title, or id) and a direction toggle
  (ascending/descending).
- The frontend ordering logic in `ChannelsView` SHALL become configurable:
  `last_date` treats channels without episodes as the oldest, `title`
  sorts case-insensitively, and `id` sorts numerically.
- The selected sort key and direction SHALL persist across reloads.
- No API or backend change: `GET /api/1.0/channels/` already carries `last_date`,
  and all sorting stays in the frontend.

## Capabilities

### New Capabilities
<!-- None: sorting already lives in the frontend under `channels-list-ordering`. -->

### Modified Capabilities
- `channels-list-ordering`: the frontend ordering requirement is extended from a
  fixed last-episode sort to a configurable sort key (last episode, title, id)
  with an adjustable direction, exposed through controls in the channels view.

## Impact

- `frontend/src/lib/utils/channel.sort.ts`: new pure module with `sortChannels`
  and the `ChannelSortKey` / `SortDirection` types.
- `frontend/src/lib/utils/channel.sort.test.ts`: unit tests for the comparator.
- `frontend/src/views/ChannelsView.vue`: `sortedChannels` becomes configurable
  and the view gains sort controls near the search box; the choice is persisted
  in `localStorage`.
- No schema migration, no backend routes, no change to the channel payload.
