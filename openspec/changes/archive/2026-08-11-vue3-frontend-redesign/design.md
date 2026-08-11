## Context

The app is a Rust backend (actix-web 4, actix-session, sqlx/sqlite) serving a static SvelteKit SPA from `frontend/`. The frontend currently uses SvelteKit with Flowbite and does not match the new Stitch design (4 screens: Login, Channels, Chapters, Add channel dialog). The Stitch HTML references are internally inconsistent (3 palettes, 2 font families, 2 icon sets) and the user wants a single unified design system based on the project's existing brand colors (coral `#fe795d` + sky `#0ea5e9`) with light/dark themes, reimplemented as a Vue 3 SPA.

The backend API contract is stable and must not change: `POST /api/1.0/login/`, `GET /api/1.0/logout/`, `GET|POST /api/1.0/channels/`, `PUT|DELETE /api/1.0/channels/{slug}/`, `GET /api/1.0/channels/{id}/episodes/`, `GET /api/1.0/config/`. Auth is cookie-session based; the SPA relies on the API's `user` field / `401` responses for auth state.

## Goals / Non-Goals

**Goals:**
- Replace the SvelteKit frontend with a Vue 3 SPA (Vite + Vue Router + Pinia + Tailwind v4).
- One unified design system (tokens, themes, primitives) across all four screens.
- Light + dark themes with a manual, persisted toggle.
- Faithful, responsive re-implementation of the four Stitch HTML screens.
- Preserve existing behavior: live search, pagination, channel CRUD, auth redirect flow.
- Keep the backend contract and the static-file serving model unchanged.

**Non-Goals:**
- No backend (Rust) changes — API contract, DB schema, and auth mechanism stay as-is.
- No new REST endpoints, no SSO, no multi-user support.
- No migration of the Svelte codebase — it is rewritten, not ported incrementally.
- No streaming audio player redesign beyond the current episode player controls.
- No i18n, no PWA offline support.

## Decisions

### D1: Stack — Vue 3 + Vite + Vue Router + Pinia + Tailwind v4

Standard, well-supported Vue 3 toolchain. Vite is the de-facto build tool (user already uses it elsewhere). Pinia provides typed stores for auth, theme, notifications, and channels. Tailwind v4 is already used by the current frontend, keeping CSS conventions familiar.

**Alternatives considered:** Svelte 5 (keep current family) — rejected, user explicitly wants Vue 3; Nuxt — rejected, no SSR needed (static files).

### D2: No full UI component library — Radix-Vue primitives only

The four screens are custom-designed; full libraries (Element Plus, PrimeVue, Naive UI, Flowbite Vue) impose their own visuals that fight the Stitch design. The only widgets with real behavior are: dialog (add/edit channel), toggle switch (active), inputs, buttons, pagination, notifications. These are cheap to build with Tailwind + a handful of headless Radix-Vue primitives (Dialog, Switch) for a11y and focus management.

**Alternatives considered:** Element Plus / Naive UI (batteries included, themable) — rejected for visual lock-in and bundle weight; Flowbite Vue (continuity with current app) — rejected, its default styles clash with the new design system.

### D3: Icons — Phosphor

Single icon set replaces the mixed Material Symbols / Phosphor / inline SVG in the Stitch HTML. Phosphor has a Vue package (`@phosphor-icons/vue`), tree-shakeable, consistent stroke weight.

**Alternatives considered:** Material Symbols (used in Channels HTML) — heavier, Google dependency; inline SVG (Login/Add-dialog HTML) — not scalable across screens.

### D4: Design tokens via Tailwind v4 `@theme` + CSS variables

Both themes defined as CSS variables (e.g. `--surface`, `--surface-card`, `--primary`, `--accent`, `--text`, `--text-muted`, `--border`), wired into Tailwind v4 `@theme` so components use semantic utility classes (`bg-surface-card`, `text-primary`, etc.). Dark mode via `class="dark"` on `<html>` with the `@custom-variant dark` pattern already used in `app.css`. No runtime theming library.

### D5: Theme state in a Pinia store + localStorage + `prefers-color-scheme`

Theme store holds `'light' | 'dark'`. On init: read `localStorage.theme`; if absent, use `matchMedia('(prefers-color-scheme: dark)')`, defaulting to dark. Set/remove `dark` class on `<html>`. Toggle button in the app header calls the store. Inline script in `index.html` applies the saved theme before Vue mounts to avoid flash-of-wrong-theme (FOUC).

### D6: Client-side auth with Vue Router guards

Auth store holds user state (`user` from API responses). Router `beforeEach` guard: protected routes (`/`, `/:channelId`) require `auth.isAuthenticated`, else `redirect('/login?next=' + to.fullPath)`; `/login` redirects to `/` when authenticated. API client detects `401` / `user: null` in responses and clears auth, then redirects to `/login`. `next` param restores the destination after login (preserves current behavior from `+page.ts`).

**Session restoration on reload:** on app bootstrap, `main.ts` calls `GET /api/1.0/session/` (an existing backend endpoint) and restores the user into the auth store **before** `app.mount('#app')`. This ensures the router's first navigation runs with the auth state already populated, so a page reload does not bounce an authenticated user back to `/login`.

**Alternatives considered:** Keep SvelteKit-style server redirect — not possible in a pure SPA without SSR; middleware — none available in Vite static serving; restore session lazily on first API call — rejected because the router guard runs before the first API call and would redirect prematurely.

### D7: Screens map 1:1 to routes/components

| Route | Screen | Components |
|---|---|---|
| `/login` | Login | `LoginView.vue`, `AppCard` (glass) |
| `/` | Channels | `ChannelsView.vue`, `AppHeader`, `SearchInput`, `ChannelCard.vue`, `AddChannelDialog.vue`, `ConfirmDialog.vue`, `Pagination.vue` |
| `/:channelId` | Chapters | `EpisodesView.vue`, `AppHeader`, `SearchInput`, `EpisodeCard.vue` (with player controls) |

Shared: `AppHeader.vue` (brand, search on desktop, create button, theme toggle, logout, avatar), `AppNotification.vue`, `AppLoading.vue`. State: `auth`, `theme`, `channels`, `notifications`.

### D8: API client module

Thin typed wrapper around `fetch` (`src/lib/api/`) with `baseEndpoint` (dev `http://localhost:6996`, prod same-origin). Methods: `login`, `logout`, `getChannels`, `createChannel`, `updateChannel`, `deleteChannel`, `getEpisodes`, `getConfig`. Same JSON contract and error shape as current frontend.

### D9: Build stays static — same Docker pipeline

SvelteKit `adapter-static` output (`/app/build`) replaced by Vue `vite build` output (`dist/`). Dockerfile frontend stage: `pnpm install && pnpm test && pnpm run build`, then copy the dist directory into `/app/html`. Version sync mechanism (Cargo.toml + package.json + docker-bake.hcl) unchanged.

### D10: Content images from real data only

Channel covers bind to `channel.image`, episode thumbnails to `episode.image`, avatar is omitted or a generic icon (no `aida-public` placeholder images). Login background is the CSS radial-gradient from the Login HTML; Add-dialog overlay uses the same gradient, not the dashboard image.

### D11: Channel metadata extraction uses a browser-like User-Agent

`src/models/ytinfo.rs` fetches the channel URL with `ureq` sending a browser-like `User-Agent` and `Accept-Language` (via `.header()` — ureq 3.x API). Without these, YouTube can serve a block page with no `og:*` metadata, producing empty `title`/`image`. A channel whose URL fails to resolve is still created with empty metadata and the channel card shows a placeholder mic icon instead of a missing image.

### D12: Runtime paths resolve by environment for local dev

The backend previously hardcoded Docker container paths (`/app/html`, `/app/audios`, `/app/.local/bin/yt-dlp`, `cookies-cp.txt`), which do not exist on a local machine and caused the episode worker to fail silently (empty episode lists). These are now resolved at runtime by environment-detecting helpers in `src/models/config.rs`, keeping Docker behavior unchanged:

- `html_path` (config field): `/app/html` in Docker, else `frontend/dist`.
- `audios_dir()`: `/app/audios` in Docker, else `audios` (relative to the project).
- `ytdlp_path()`: `/app/.local/bin/yt-dlp` in Docker, else `yt-dlp` (from PATH).
- `cookies_file()`: `cookies-cp.txt` if present, else empty (omits the `--cookies` flag).

The episode worker (`src/utils/worker.rs`) receives these resolved paths as parameters (`folder`), and the media service and channel-delete handler use `audios_dir()`. This lets the app run locally without sudo while the Docker build still copies `dist/` into `/app/html` and works unchanged.

## Risks / Trade-offs

- **FOUC between themes** → Inline script in `index.html` sets the `dark` class before Vue mounts; theme store reads the same source of truth.
- **Auth state divergence between router guard and API** → Single source of truth: auth store; API client is the only writer (clears on `401`), router guard only reads it.
- **Tailwind v4 + custom tokens learning curve** → Small token surface (~10 variables) kept explicit in `app.css`; primitives are thin wrappers.
- **Radix-Vue adds dependency weight** → Only Dialog/Switch pulled in; alternative is hand-rolled but risks losing focus-trapping/a11y.
- **Placeholder images in Stitch HTML leak into implementation** → Spec requires reading Stitch only for structure; a checklist in tasks enumerates the aida-public URLs to never copy.
- **Rewriting, not porting, loses working code** → Existing utility helpers (list filtering, formatters, validation) are ported into `src/lib/utils/` rather than re-invented.
