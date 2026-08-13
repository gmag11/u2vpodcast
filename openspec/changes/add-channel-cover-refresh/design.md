## Context

The frontend is a Vue 3 SPA (`frontend/src`) served as static files, talking to a Rust actix-web backend. `ChannelCard.vue` renders each channel with a cover image sourced from `channel.image`. That URL is fetched once at channel creation: `Channel::new` calls `YTInfo::new(&url)` which parses the YouTube `og:image` meta tag (stripping query-string sizing params) and stores it in the `channels.image` column.

Today the only way to re-read the cover is `POST /channels/{channel}/update/`, which triggers a full episode refresh (yt-dlp fetch + audio downloads) — far heavier than needed for a cover rebrand fix.

## Goals / Non-Goals

**Goals:**
- Let an authenticated user re-read the YouTube cover image URL for a single channel on demand.
- Persist the new URL to `channels.image` and return the updated channel so the SPA can re-render.
- Provide a small button on each channel card with loading state and success/error feedback.
- Keep the operation lightweight (single HTTP fetch + one DB update, no episode work).

**Non-Goals:**
- Re-fetching channel title/description (only the image).
- Refreshing covers in bulk or on a schedule.
- Downloading or storing the image bytes (URL only, matching current behavior).
- Backfilling covers for channels that have none.

## Decisions

### 1. New lightweight endpoint `POST /api/1.0/channels/{id-or-slug}/image/`

Adds a dedicated endpoint that re-reads the cover for one channel. Uses the existing id-or-slug resolution (`Channel::read_by_id_or_slug`) consistent with the other channel routes. Response mirrors other handlers: `CResponse::ok(session, channel)` with the updated channel.

**Alternative considered:** reusing `POST .../update/`. Rejected — it triggers full episode refresh; unacceptable cost for a cover change.

### 2. Run the re-fetch synchronously in the handler

`YTInfo::new` is a single `ureq` HTTP GET plus regex parse, then one `UPDATE ... RETURNING *`. Fast enough to run inline so the client gets the updated channel in the same request-response round trip and the card can update immediately.

**Alternative considered:** background task (like `update_episodes`). Rejected — no long downloads, and the button wants the fresh image synchronously.

### 3. New model method `Channel::update_image`

Adds a focused model method that re-fetches `YTInfo` for the channel URL and runs `UPDATE channels SET image = $1, updated_at = $2 WHERE id = $3 RETURNING *`. Reuses existing `get_image`/`YTInfo` logic. If the fetch or update fails, it returns `Error`, the handler surfaces it, and the stored image is left untouched.

### 4. Frontend: emit event from ChannelCard, handle in ChannelsView

`ChannelCard.vue` gets a small image-refresh button (icon button matching the existing Edit/Delete pattern, `@click.stop` so it does not navigate). It emits a `cover-refresh` event with the channel. `ChannelsView.vue` listens, calls the new `api.refreshChannelImage(slug)`, replaces the channel in the `channels` array with the returned one, and shows a success/error notification via `useNotificationStore` (same pattern as update/delete).

**Alternative considered:** doing the fetch inside the card. Rejected — cards stay presentational; the view owns data mutations and the shared channel list, consistent with the existing `update`/`delete` event flow.

### 5. Client loading state

The card tracks a `refreshing` boolean per card; the button disables and swaps to a spinner/disabled state while the request is in flight. Prevents double clicks on the same card.

### 6. Distinguish cover refresh from episode refresh

The cover button uses a distinct image/cover icon (`PhImage`) instead of the reload arrows used by the episode-refresh control in `EpisodesView.vue`, and shows a CSS-only tooltip ("Reload cover") on hover via a `group relative` wrapper. A custom Tailwind tooltip was chosen over the radix-vue `Tooltip` component (already a dependency but unused elsewhere in the SPA) to keep the change minimal and consistent with the existing hand-rolled styling.

## Risks / Trade-offs

- [YouTube blocks the plain `ureq` fetch] → Mitigation: `YTInfo::new` already sends a browser-like User-Agent and Accept-Language; on failure the handler returns an error and the old image is preserved.
- [Button hit area too close to Edit/Delete] → Mitigation: `@click.stop`, distinct image icon, `aria-label`, disabled state while loading.
- [Tooltip hidden under card edge when card is near the top] → Mitigation: tooltip is positioned above the button (`bottom-full`), which sits in the card footer; overflow is not clipped by the card, so it remains visible.
- [Slow network makes button feel unresponsive] → Mitigation: loading state shown immediately on click; existing notification store surfaces completion.
- [No image re-fetch for empty covers] → Mitigation: deliberately out of scope; endpoint works for any channel, empty `image` just stays empty on failure.

## Migration Plan

No DB schema change (`image` column already exists). Deploy is a normal backend + SPA build; no data migration or rollback concerns beyond reverting the code.
