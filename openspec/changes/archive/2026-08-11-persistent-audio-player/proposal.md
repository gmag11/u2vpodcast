## Why

The episode player currently lives inside each `EpisodeCard` with its own private `<audio>` element and local state. Playback is lost as soon as the user scrolls, navigates away, or wants to keep listening while browsing. There is no way to keep playing across the app or to control playback from a single, always-available surface.

## What Changes

- Introduce a shared, app-wide audio player store (Pinia) that owns the single `<audio>` element and the current episode (source, title, thumbnail, slug, yt_id).
- Add a **persistent player bar** rendered at the bottom of the screen, full width, overlaying the page content, showing the episode thumbnail, title, and the standard controls (play/pause, stop, position scrubber, volume, playback speed).
- Rewire `EpisodeCard` so its per-card controls drive the same shared store instead of a private `<audio>` element. Starting playback in a card starts the shared player; starting another episode swaps the shared source.
- Auto-hide: when playback is stopped (via stop or audio end), the persistent player slides down and disappears after a 10-second delay; a smooth downward animation removes it. It reappears on any new play action.
- Keep the existing `vue3-spa` and `unified-design-system` specs' behaviors intact (same media URL pattern, same visual tokens/icons).

## Capabilities

### New Capabilities
- `persistent-audio-player`: Shared app-wide audio playback — a Pinia store owning the single `<audio>` element and current episode, a persistent bottom player bar with full controls, bidirectional synchronization between the per-card controls and the persistent bar, episode swapping on new play, and animated auto-hide 10 seconds after playback stops.

### Modified Capabilities
- `vue3-spa`: The episode player requirement changes from a per-card, card-scoped player to a shared app-wide player — the `EpisodeCard` no longer owns its own `<audio>` element and instead binds to the global player store; the app shell (`App.vue`) renders the persistent player.

## Impact

- **Frontend only.** `frontend/src/stores/` gains a new audio player store; `frontend/src/components/` gains a persistent player component and `EpisodeCard.vue` is refactored to bind to the shared store; `App.vue` renders the persistent player; `EpisodesView.vue` is unaffected structurally.
- No backend or API changes — media continues to be served from `/media/{slug}/{yt_id}.mp3` with the existing range-request behavior.
- No design-system changes — the player reuses the existing tokens, Phosphor icons, and the `toHHMMSS` formatter.
