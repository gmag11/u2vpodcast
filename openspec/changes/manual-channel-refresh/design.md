## Context

The backend (actix-web, sqlx/sqlite) refreshes channels in `src/utils/worker.rs::do_the_work`, spawned from `main.rs` in a loop that sleeps `sleep_time` hours between cycles. `process_channel(pool, channel, ytdlp, folder)` and `clean_channel(...)` are private and only run inside that loop. There is no way to update a single channel on demand, and new channels wait for the next cycle.

The frontend is a Vue 3 SPA. `EpisodesView.vue` lists episodes for one channel; `ChannelsView.vue` creates channels via `api.createChannel`. The API client lives in `frontend/src/lib/api/client.ts`; loading and notification feedback use the existing Pinia stores.

## Goals / Non-Goals

**Goals:**
- Backend endpoint to refresh a single channel on demand (out of cycle).
- Auto-refresh a newly created channel immediately.
- Frontend "Refresh" button on the episodes page with loading + notification feedback.
- Reuse the existing yt-dlp download, episode storage, and clean-up logic.

**Non-Goals:**
- No changes to the periodic worker cycle itself (it stays as-is).
- No queueing/deduplication system beyond fire-and-forget spawns.
- No UI on the channels dashboard for bulk refresh.
- No progress reporting from the background refresh to the UI (only start/acknowledgment).

## Decisions

### D1: Expose a public single-channel refresh in worker.rs

Rename/generalize the per-channel work into a public async function, e.g. `pub async fn update_channel(pool: &SqlitePool, channel_id: i64) -> Result<(), Error>` that reads the channel row, constructs `Ytdlp` and the audios folder from the environment helpers (`ytdlp_path()`, `cookies_file()`, `audios_dir()`), and calls the existing `process_channel` + `clean_channel` for that single channel. `do_the_work` keeps looping over all channels and can reuse the same helper.

**Alternatives considered:** Duplicating the logic in the handler — rejected, keeps one source of truth for refresh behavior.

### D2: Refresh runs as a background task, request returns immediately

The handler spawns the work on the actix runtime (`actix_web::rt::spawn`) with a cloned `SqlitePool`, then returns `CResponse::ok` immediately. Downloads (yt-dlp) can take minutes; blocking the request would time out clients.

**Alternatives considered:** Awaiting the full download in the handler — rejected, too slow for a UI action; a message-queue/worker crate — rejected, overkill for this scale.

### D3: New endpoint + registration

Add `#[post("/channels/{channel}/update/")]` in `src/handlers/channels.rs`. It resolves the channel by slug via `Channel::read_by_id_or_slug`, and on success spawns `update_channel(pool.clone(), channel.id)`. Register it in `src/handlers/mod.rs` inside the `RequireSession` scope alongside the other channel routes, so it is protected like the rest of the API.

**Security note:** it sits behind `RequireSession`, matching the existing `route-protection` contract.

### D4: Auto-refresh on create

In the `create` handler, after `Channel::new` succeeds, spawn `update_channel(pool.clone(), channel.id)` the same way. The create response is unchanged.

**Alternatives considered:** Frontend-triggered update after create — rejected, backend-atomic is simpler and always runs regardless of client behavior.

### D5: Frontend API client + EpisodesView refresh button

Add `api.refreshChannel(slug)` to `client.ts` issuing `POST /api/1.0/channels/{slug}/update/`. In `EpisodesView.vue`, add an `AppButton` (secondary/ghost, refresh icon) in the page header or next to the search bar, bound to the current channel slug (the route param resolves via the episodes list's `channel_slug`). On click: set `refreshing=true`, call the endpoint, show a success notification ("Channel update started"), handle errors with an error notification, clear `refreshing`.

**Slug resolution:** episodes carry `channel_slug`; the page derives the slug from the first episode (existing pattern) or from a channel lookup. If no episodes exist yet, fall back to a channel fetch by id (or disable the button until the slug is known).

## Risks / Trade-offs

- **Concurrent refreshes of the same channel** (manual + periodic) → Acceptable: `process_channel` skips already-downloaded `yt_id`s (`episode_exists`), so duplicate work is idempotent; two yt-dlp runs may overlap but not corrupt rows.
- **Long-running spawn holds DB pool** → Cloned pool, one channel at a time; the periodic loop may interleave, which is fine given idempotency.
- **Button disabled until slug known** → Derive slug from episodes; if empty, fetch channel by id before enabling, or keep disabled with a tooltip.
- **No progress feedback** → By design; notification confirms the request started, not that downloads finished.
