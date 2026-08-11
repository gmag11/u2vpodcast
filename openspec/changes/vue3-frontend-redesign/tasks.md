## 1. Project setup

- [ ] 1.1 Scaffold Vue 3 app in `frontend/` with Vite (vue-ts template), replacing the SvelteKit project files; keep `package.json` name/version and `pnpm` as the package manager
- [ ] 1.2 Install deps: vue, vue-router, pinia, tailwindcss v4 + @tailwindcss/vite, @radix-ui-vue, @phosphor-icons/vue, vitest
- [ ] 1.3 Configure Vite (`vite.config.ts`): base path for static serving, Tailwind plugin, Vue plugin, test config
- [ ] 1.4 Wire `index.html`: title, Geist + Inter font links, inline theme-init script (localStorage → prefers-color-scheme) to avoid theme FOUC
- [ ] 1.5 Configure Tailwind v4 in `src/app.css`: `@import 'tailwindcss'`, `@custom-variant dark`, `@theme` mapping token CSS variables to utilities

## 2. Design system (unified-design-system)

- [ ] 2.1 Define token CSS variables for light and dark themes in `app.css` (`--surface`, `--surface-card`, `--surface-input`, `--text`, `--text-muted`, `--border`, `--primary` coral scale, `--accent` sky scale, `--success`, `--error`) following the existing brand palette from `frontend/src/app.css`
- [ ] 2.2 Add semantic Tailwind utilities (`bg-surface-card`, `text-primary`, `border-outline`, etc.) backed by the tokens
- [ ] 2.3 Create theme Pinia store: `'light' | 'dark'`, init from localStorage/prefers-color-scheme, set/remove `dark` class on `<html>`, `toggle()` + `apply()`
- [ ] 2.4 Create shared primitives with Tailwind + tokens: `AppButton.vue` (primary/secondary/ghost variants), `AppInput.vue`, `SearchInput.vue`, `AppCard.vue` (glass-card), `AppToggle.vue` (switch), `AppDialog.vue` (Radix-Vue Dialog wrapper)
- [ ] 2.5 Create `AppHeader.vue`: brand (AuraPod-style mic icon + wordmark), desktop search slot, theme toggle, logout, avatar (generic icon, no placeholder image)
- [ ] 2.6 Create `AppNotification.vue` + `AppLoading.vue` wired to Pinia stores

## 3. API client and auth

- [ ] 3.1 Create `src/lib/api/` client with `baseEndpoint` (dev `http://localhost:6996`, prod same-origin) and typed methods: `login`, `logout`, `getChannels`, `createChannel`, `updateChannel`, `deleteChannel`, `getEpisodes`, `getConfig`
- [ ] 3.2 Define TS types (`Channel`, `Episode`, `User`, `Response`, `CustomError`) mirroring `frontend/src/lib/types.ts`
- [ ] 3.3 Port existing utility helpers into `src/lib/utils/`: list word filtering, formatters, input validation
- [ ] 3.4 Create auth Pinia store: holds `user`, `isAuthenticated`; set from login response; `clear()` on logout or `401`
- [ ] 3.5 Add API client 401/user:null handling that clears auth and triggers redirect to `/login`
- [ ] 3.6 Create `src/lib/router/index.ts` with routes `/`, `/login`, `/:channelId` and a `beforeEach` guard: protected routes redirect to `/login?next=...` when unauthenticated; `/login` redirects to `/` when authenticated

## 4. Login screen

- [ ] 4.1 Implement `LoginView.vue` following Stitch Login HTML (`9dba5a04`): radial-gradient background, centered glass-card, logo, username + password fields, coral primary submit button, error display
- [ ] 4.2 Wire form submit to `api.login`, loading indicator, success notification, post-login navigation to `next` or `/`

## 5. Channels dashboard

- [ ] 5.1 Implement `ChannelsView.vue` following Stitch Channels HTML (`90b4d6eb`): AppHeader (fixed, backdrop-blur), page header ("Dashboard"), responsive 3/2/1-column channel card grid
- [ ] 5.2 Implement `ChannelCard.vue`: cover image bound to `channel.image`, title, description (line-clamp), actions (feed link, edit, delete) with icon hover states
- [ ] 5.3 Implement `AddChannelDialog.vue` following Stitch Add-dialog HTML (`d2b08930`): modal with active toggle, Url input, Max number of episodes, First episode date, Create channel / Cancel actions
- [ ] 5.4 Implement `ConfirmDialog.vue` for delete confirmation; wire channel create/update/delete through the API client
- [ ] 5.5 Implement pagination (per-page from `/api/1.0/config/`, default 3), page query param, prev/next + numbered pages

## 6. Chapters (episodes) screen

- [ ] 6.1 Implement `EpisodesView.vue` following Stitch Chapters HTML (`95dd5ddc`): search bar, episode list, no-results message
- [ ] 6.2 Implement `EpisodeCard.vue`: thumbnail bound to `episode.image`, title, date, description, YouTube link (sky accent), and player controls (play button, progress bar, volume/duration) per the Stitch reference
- [ ] 6.3 Wire episodes fetch (`/api/1.0/channels/{id}/episodes/`) and live search filtering

## 7. Responsive and polish

- [ ] 7.1 Verify responsive behavior on all screens (grid collapse, nav search hidden on mobile, episode card stacking, dialogs centered) against the responsive spec scenarios
- [ ] 7.2 Run `pnpm lint` (prettier + eslint) and `pnpm check` (vue-tsc) clean; add/extend unit tests for filtering helpers and auth guard logic with vitest

## 8. Build and Docker integration

- [ ] 8.1 Update `Dockerfile` frontend stage: `pnpm install --frozen-lockfile && pnpm test && pnpm run build`, copy Vue `dist/` output into `/app/html` (replacing SvelteKit `build/`)
- [ ] 8.2 Update `frontend/.gitignore`, remove SvelteKit-specific configs (`svelte.config.js`, adapter deps), update `frontend/README.md`
- [ ] 8.3 Keep version sync: confirm `frontend/package.json` version matches `Cargo.toml` and `docker-bake.hcl` tag
- [ ] 8.4 Build the frontend in the Docker pipeline (`pnpm run build`) and smoke-test the served static SPA against the API
