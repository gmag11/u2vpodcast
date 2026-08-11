# u2vpodcast frontend

Vue 3 single-page application served as static files by the u2vpodcast backend at `/app`.

## Stack

- Vue 3 + TypeScript
- Vite (build, base path `/app`)
- Vue Router (SPA routing with auth guards)
- Pinia (state: auth, theme, notifications, loading)
- Tailwind CSS v4 (design tokens, light/dark themes)
- Radix-Vue (headless primitives: dialog, switch)
- Phosphor icons
- Vitest (unit tests)

## Design

The UI follows the Stitch design reference (project "Modern Podcast Dashboard Redesign"),
unified into a single design system: coral primary, sky accent, light/dark themes,
Geist + Inter typography. See `openspec/changes/vue3-frontend-redesign/` for the full spec.

## Developing

```bash
pnpm install
pnpm dev
```

Dev mode proxies API calls to `http://localhost:6996` (the Rust backend).

## Checking

```bash
pnpm lint      # prettier + eslint
pnpm check     # vue-tsc type check
pnpm test      # vitest
```

## Building

```bash
pnpm run build
```

Outputs to `dist/`, which the Dockerfile copies into `/app/html`.
