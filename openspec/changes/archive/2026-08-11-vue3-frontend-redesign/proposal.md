## Why

The current frontend is a SvelteKit SPA styled with Flowbite, whose look does not match the new design produced in Stitch (4 screens: Channels, Chapters, Login, Add Channel dialog). The Stitch designs are inconsistent across screens (3 palettes, 2 fonts, 2 icon sets) and must be unified into one coherent design system before implementation. The user wants a modern Vue 3 SPA that faithfully follows the Stitch reference screens.

## What Changes

- **BREAKING**: Replace the SvelteKit frontend with a Vue 3 SPA (Vite + Vue Router + Pinia + Tailwind CSS v4).
- Introduce a unified design system derived from the existing brand palette (coral `#fe795d` primary, sky `#0ea5e9` accent) with **light and dark themes** and a manual theme toggle persisted in localStorage.
- Rebuild the 4 screens from the Stitch HTML references with responsive layout: Login, Channels (dashboard), Chapters (episode list with player controls), Add Channel modal.
- Keep the backend API contract unchanged (Rust/actix-web): `/api/1.0/login/`, `/api/1.0/logout/`, `/api/1.0/channels/`, `/api/1.0/channels/{slug}/`, `/api/1.0/channels/{id}/episodes/`, `/api/1.0/config/`.
- Replace session-aware server-side loading (SvelteKit `+page.ts` redirect flow) with client-side Vue Router guards that read the 401/`user: null` API response and redirect to `/login`.
- Preserve existing client-side behavior: live word-based search on channels and episodes, pagination, channel create/edit/delete dialogs.
- Do NOT use the placeholder images from the Stitch HTML (example covers, avatar, thumbnails); use real `channel.image` / `episode.image` data and CSS backgrounds instead.
- Update the Dockerfile build pipeline from SvelteKit to Vue (`vite build` output served as static files) and keep the version sync (Cargo.toml + package.json + docker-bake.hcl) unchanged in mechanism.

## Capabilities

### New Capabilities
- `vue3-spa`: Vue 3 single-page application replacing the SvelteKit frontend — app shell, routing, API client, auth guards, state management.
- `unified-design-system`: Coherent visual language across all screens — tokens, light/dark themes, typography, spacing, reusable UI primitives (cards, inputs, buttons, dialogs, toggle).

### Modified Capabilities
- `list-search`: Client-side live search requirements now apply to the Vue SPA screens (channels homepage and episodes page) instead of the SvelteKit implementation.
- `route-protection`: The anonymous-user redirect flow moves from server-side SvelteKit loading to client-side Vue Router guards reacting to the API `401`/`user: null` response; the backend 401 contract itself is unchanged.

## Impact

- **Frontend**: `frontend/` directory rewritten from SvelteKit to Vue 3 (Vite, Vue Router, Pinia, Tailwind v4, headless Radix-Vue primitives, Phosphor icons). Existing Svelte components, stores, and routes removed.
- **Build/deploy**: `Dockerfile` frontend stage switches from `pnpm run build` (SvelteKit static adapter) to Vue `vite build`; static output dir changes from `/app/build` to Vue's dist output.
- **Backend**: No Rust code changes expected; API contract preserved. Auth remains cookie-session based (actix-session).
- **Versioning**: `frontend/package.json` version must stay in sync with `Cargo.toml` and `docker-bake.hcl` tag (mechanism unchanged).
- **Design assets**: Stitch HTML references used as the source of truth; placeholder `aida-public` images are excluded from the final UI.
