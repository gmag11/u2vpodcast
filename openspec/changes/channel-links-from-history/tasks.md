## 1. Make compact channel label a link

- [x] 1.1 In `frontend/src/components/EpisodeCard.vue`, replace the compact channel-title `<p>` (guarded by `compact && props.episode.channel_title`) with a `<RouterLink>` targeting `{ name: 'episodes', params: { channelId: String(props.episode.channel_id) } }`
- [x] 1.2 Keep the existing label styling (`text-xs font-medium uppercase tracking-wide text-accent-500`) and add `hover:underline` for click affordance
- [x] 1.3 Confirm the default (non-compact) card variant renders no channel link and the card's play/stop controls are unchanged

## 2. Verify

- [x] 2.1 Build and lint the frontend (`pnpm run lint` and `pnpm run build` in `frontend/`)
- [x] 2.2 Verify in `HistoryView`: clicking the channel name navigates to that channel's episode list, and playback state is preserved when audio is active
