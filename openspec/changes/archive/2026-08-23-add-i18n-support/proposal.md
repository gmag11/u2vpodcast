## Why

The SPA frontend is 100% hardcoded English: every heading, button, placeholder, toast notification, empty state, and `aria-label` is a literal string scattered across ~20 Vue components and views. The project's audience includes non-English speakers (the author's own community is Spanish-speaking), and there is no language infrastructure whatsoever. Adding internationalization makes the UI serve Spanish and English (with room to grow) without duplicating components.

## What Changes

- Introduce `vue-i18n` (v11, composition API mode) as the SPA's translation layer.
- Add two locale catalogues, `en` (baseline, current English UI) and `es` (Spanish translation), under `frontend/src/i18n/locales/`.
- Add a Pinia `locale` store mirroring the existing `theme` store: persists the user's choice in `localStorage`, falls back to `navigator.language` on first visit, and defaults to `en` when the browser locale is unsupported.
- Add a language switcher control in `AppHeader` (desktop bar and mobile drawer), listing available languages by their autoglottonym (`English`, `Español`), switching reactively without a page reload.
- Migrate all user-facing UI strings in the four views and sixteen components to translation keys: text nodes, button labels, input placeholders, toast/notification messages, empty states, tooltips, and `aria-label` accessibility attributes.
- Make date rendering locale-aware: `EpisodeCard` currently hardcodes `toLocaleDateString('en-US')`; it will format with the active locale.
- Localize the dynamic channel sync tooltip (`Updated 5h ago. Status: Ok`) using i18n interpolation.
- Update the two existing component tests so they mount with the i18n plugin and continue asserting against the `en` locale.
- **No backend changes.** API responses only carry HTTP status strings (`"400 Bad Request"`), not human-readable sentences; RSS feed content (titles, descriptions) is YouTube data and iTunes metadata values (`Full`, `No`) are RSS spec enums, neither translatable.

## Capabilities

### New Capabilities

- `i18n`: Frontend internationalization — locale detection and persistence, reactive language switching, translation key infrastructure, locale-aware date formatting, and complete externalization of user-facing strings.

### Modified Capabilities

<!-- No existing spec-level behavior changes; existing capabilities keep their functional requirements. -->

## Impact

- **Frontend**: `frontend/src/i18n/` (new: index + `locales/en.json`, `locales/es.json`), `frontend/src/stores/locale.ts` (new), `frontend/src/main.ts` (register plugin), `frontend/src/router/` (unchanged), all 4 views and 16 components in `frontend/src/views/` and `frontend/src/components/`, and the 2 existing component tests in `frontend/src/components/` and `frontend/src/views/`.
- **Dependencies**: adds `vue-i18n` (^11) to `frontend/package.json`.
- **Backend**: none. No Rust changes, no schema changes, no config changes.
- **Storage**: `localStorage` key `locale` added (alongside the existing `theme` key); no database changes.