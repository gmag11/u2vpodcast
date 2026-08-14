## 1. Simplify episode card controls

- [x] 1.1 Remove the inline seek bar block (progress slider) from `frontend/src/components/EpisodeCard.vue`
- [x] 1.2 Remove the playback speed control (gauge button + speed dropdown) and the volume control (mute button + range input) rows from `EpisodeCard.vue`
- [x] 1.3 Remove now-unused script code: `showSpeed`, `speeds`, `onSeek`, `onVolumeInput`, and the `PhGauge`, `PhSpeakerHigh`, `PhSpeakerSlash` imports
- [x] 1.4 Change the duration label to show total duration only: `player.durationLabel` when the episode is current, else `props.episode.duration`; remove the `player.currentLabel` position counter

## 2. Reposition play/stop buttons

- [x] 2.1 Add a mobile placement (below the `sm` breakpoint): play/pause and stop buttons rendered to the right of the thumbnail in a horizontal strip, with the thumbnail fixed-width instead of full-width
- [x] 2.2 Add a desktop placement (`sm+`): play/pause and stop buttons rendered below the thumbnail in a vertical column, hidden on mobile
- [x] 2.3 Ensure only one button placement is visible per viewport width and both placements use identical button markup, aria-labels, disabled states, and shared-store bindings

## 3. Verify

- [x] 3.1 Confirm `PersistentPlayer.vue` and `frontend/src/stores/player.ts` are unchanged
- [x] 3.2 Build the frontend (e.g. `npm run build` in `frontend/`) to confirm no unused imports/errors
- [x] 3.3 Manually verify in both `EpisodesView.vue` and `HistoryView.vue`: no seek/speed/volume/position-counter on cards, total duration shown, play/stop below thumbnail on desktop and right of thumbnail on mobile, and the persistent bar still provides scrubber/volume/speed
