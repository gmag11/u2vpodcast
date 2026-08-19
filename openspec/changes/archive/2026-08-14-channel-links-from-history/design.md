## Context

`EpisodeCard.vue` renders two variants. The compact variant (used by `HistoryView`) shows a small uppercase channel label above the episode title. Today that label is a static `<p>` with `text-accent-500`. The app already has an episodes route named `episodes` at `/app/:channelId(\d+)` that resolves the channel by numeric id and lists its episodes (`EpisodesView.vue`). `ChannelCard.vue` already navigates to it via `router.push({ name: 'episodes', params: { channelId: String(channel.id) } })`. The `Episode` type carries `channel_id: number`.

Constraint: the episode list card (default variant) must not change; it has no channel label.

## Goals / Non-Goals

**Goals:**
- Make the compact card's channel label a link to the channel's episode list.
- Keep the existing label styling and spacing.

**Non-Goals:**
- No new routes, no API changes, no changes to `EpisodesView` or `HistoryView`.
- No behavior change for the default (episode list) card variant.
- No change to the persistent player or player store.

## Decisions

### 1. Use RouterLink with the existing episodes route

Replace the compact channel `<p>` with `<RouterLink>` targeting `{ name: 'episodes', params: { channelId: String(props.episode.channel_id) } }`, mirroring the navigation used in `ChannelCard.vue`. `RouterLink` is globally registered by `vue-router`, so no component import is needed.

Rationale: route-by-name avoids hardcoding the `/app/:channelId` path and stays consistent with existing navigation. The route already accepts the numeric id, and `Episode.channel_id` is always present on episodes.

### 2. Preserve label styling, add click affordance

Keep the existing classes (`text-xs font-medium uppercase tracking-wide text-accent-500`) and add `hover:underline` so the link reads as clickable, matching the YouTube link affordance already used elsewhere in the card. The label keeps the same `v-if` guard (`compact && channel_title`).

Rationale: minimal visual change; the link should look like the same label but clearly interactive.

### 3. Keep play/stop interactions independent

The link is a sibling element of the play/stop buttons, so clicking it does not trigger playback. Navigation is an SPA push (`RouterLink`); the persistent player keeps playing while the route changes.

Rationale: no custom handlers, no event propagation concerns.

## Risks / Trade-offs

- **Clicking the channel label while audio plays** keeps playback running (SPA navigation). → Acceptable; matches existing behavior when navigating via header/channel cards.
- **Channel label only exists on compact cards** (history list). → Intended scope; default cards unchanged.
