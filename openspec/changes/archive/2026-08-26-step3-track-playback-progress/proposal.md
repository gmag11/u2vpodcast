## Why

Long episodes cannot realistically be finished in one sitting. Without a stored playback position every return to an episode restarts it from zero, and there is no way to tell at a glance which episodes have already been finished. Podcast clients solve both with per-episode progress and a visible "played" mark; the web player should do the same, server-side so progress follows the user across devices.

## What Changes

- Episodes gain two persisted columns: `position_seconds` (last playback position) and `listened_at` (when the episode was completed); the existing `listen` boolean becomes the "played" mark.
- New authenticated endpoint `PUT /api/1.0/episodes/{yt_id}/progress/` accepting `{ position_seconds, listened }` updates both fields in one call. The episode is addressed by its public id (`yt_id`, the same identity used by the media URLs), so progress is associated to the individual episode regardless of how it is played (single or from a playlist).
- The player saves position throttled every ~10s while playing, and again on pause, on stop, and on `ended` / page unload.
- Starting playback resumes automatically from the stored position when it is above 30s and below ~95% of the duration; an "start over" affordance clears the position and plays from zero.
- On `ended` the episode is marked listened (`listen=true`, `listened_at=now`, position=duration).
- `EpisodeCard` shows a visible played mark (`listen=true`) and a progress hint ("Continue at MM:SS") for partially played episodes. The History screen list itself is not redefined.
- No backend change to existing GET endpoints beyond serializing the new episode fields.

## Capabilities

### New Capabilities
- `playback-progress`: per-episode persisted playback position and listened state, resumed automatically on play and reported through the episode API and player UI.

### Modified Capabilities
- `episode-cards`: episode cards display the played mark and a resume/position hint.
- `episode-persistence`: the episode entity gains `position_seconds` and `listened_at` fields plus the progress update path.

## Impact

- **Code**: `migrations/` (new ALTER TABLE on `episodes`), `src/models/episode.rs` (new fields, progress struct), `src/handlers/episodes.rs` (new PUT route registered in `src/handlers/mod.rs`), `frontend/src/lib/api/client.ts` (progress call), `frontend/src/stores/player.ts` (save/resume logic), `frontend/src/components/EpisodeCard.vue` (badge + resume hint), `frontend/src/types.ts` (episode fields).
- **APIs**: new `PUT /api/1.0/episodes/{id}/progress/`; episode GET responses gain two fields.
- **Dependencies**: none.
- **DB**: two new nullable/defaulted columns on `episodes` via sqlx migration.
- **Frontend**: player + card unit tests.