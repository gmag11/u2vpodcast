## 1. Backend: cover re-read endpoint

- [ ] 1.1 Add `Channel::update_image` model method in `src/models/channel.rs` that calls `YTInfo::new(&url)`, updates `channels.image` and `updated_at`, and returns the updated channel via `RETURNING *`
- [ ] 1.2 Add `refresh_image` handler in `src/handlers/channels.rs` for `POST /channels/{channel}/image/` that resolves the channel by id-or-slug, calls `update_image`, and returns `CResponse::ok(session, channel)`, mapping errors to error `CustomResponse`
- [ ] 1.3 Register the new route in `src/handlers/mod.rs` under the channels scope
- [ ] 1.4 Verify backend compiles and the endpoint returns the updated channel (and errors on unknown channel / failed fetch)

## 2. Frontend: API client method

- [ ] 2.1 Add `refreshChannelImage(slug)` to `frontend/src/lib/api/client.ts` calling `POST /api/1.0/channels/{slug}/image/`

## 3. Frontend: channel card button

- [ ] 3.1 Add a cover refresh button to `ChannelCard.vue` (image/refresh icon, `@click.stop`, `aria-label`) that emits a `cover-refresh` event with the channel
- [ ] 3.2 Add per-card `refreshing` state that disables the button and shows a loading indicator while in flight

## 4. Frontend: view wiring

- [ ] 4.1 In `ChannelsView.vue`, listen for `cover-refresh`, call `api.refreshChannelImage(slug)`, replace the channel in the `channels` array with the returned one, and show success/error notification via `useNotificationStore`
- [ ] 4.2 Typecheck the SPA (`npm run typecheck` if available, else `tsc --noEmit`) and verify the card updates its image on success and keeps it unchanged on failure
