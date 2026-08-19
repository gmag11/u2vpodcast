## Why

Episode cards occupy too much vertical space: each card shows a seek bar, position counter, playback speed, and volume controls that duplicate what the persistent bottom player bar already provides. Removing those controls shrinks the cards, so more episodes fit per screen.

## What Changes

- Remove the seek/scroll bar from episode cards.
- Remove the playback speed control (gauge button + speed dropdown).
- Remove the volume control (mute button + volume slider).
- Keep the total duration label only; remove the live position counter (`current / total` becomes `total`).
- Move the play/pause and stop buttons below the thumbnail on desktop (`sm+`), and to the right of the thumbnail on mobile.
- The persistent bottom player bar is NOT changed; it keeps position (scrubber), volume, and speed controls.

## Capabilities

### New Capabilities
- `episode-cards`: layout and inline playback controls of the episode card, including the reduced vertical footprint, play/pause/stop placement per breakpoint, and the total-duration-only label.

### Modified Capabilities
- `persistent-audio-player`: the "Episode card controls are bound to the shared player" requirement is updated so cards only expose play/pause/stop; seek, volume, and speed stay exclusively in the persistent bar.

## Impact

- `frontend/src/components/EpisodeCard.vue`: remove seek/speed/volume UI and the position counter; reposition play/stop buttons; keep all actions bound to the shared player store.
- No store changes (`frontend/src/stores/player.ts` unchanged; `seek`, `setVolume`, `toggleMute`, `setSpeed` remain available for the persistent bar).
- Affects both usage sites: `EpisodesView.vue` (default card) and `HistoryView.vue` (compact card).
- No backend, API, or dependency changes.
