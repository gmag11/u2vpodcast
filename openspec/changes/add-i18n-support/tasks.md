## 1. Infrastructure

- [ ] 1.1 Add `vue-i18n` (^11) to `frontend/package.json` dependencies and install with pnpm
- [ ] 1.2 Create `frontend/src/i18n/index.ts`: `createI18n({ legacy: false, fallbackLocale: 'en' })` wiring static imports of `locales/en.json` and `locales/es.json`, plus `datetimeFormats` (short/medium for `en` and `es`)
- [ ] 1.3 Register the i18n plugin in `frontend/src/main.ts` with `app.use(i18n)` before mounting
- [ ] 1.4 Create `frontend/src/stores/locale.ts` mirroring the theme store: `locale` ref, `resolveInitial()` reads `localStorage['locale']` (validated against available locales) and falls back to `navigator.language` (normalized to lang subtag, unknown → `en`), `apply()` writes localStorage, sets `document.documentElement.lang`, and assigns `i18n.global.locale.value`
- [ ] 1.5 Call `locale.init()` in `main.ts` beside `theme.init()` so locale resolves synchronously before mount

## 2. Translation catalogues

- [ ] 2.1 Build the complete `en.json` catalogue with feature-scoped keys (per design D4), covering every user-facing string inventoried across the 4 views and 16 components — text nodes, buttons, placeholders, notifications, empty states, tooltips, aria-labels, dialog copy, confirm copy, and the sync-status tooltip interpolation (`Updated {age} ago. Status: {status}`)
- [ ] 2.2 Write the `es.json` translation catalogue covering every key from `en.json`, using informal ("tú") register, with date formats for `es`
- [ ] 2.3 Add a parity unit test (e.g. `frontend/src/i18n/locales/parity.test.ts`) that asserts every key in `en.json` exists in `es.json`

## 3. Views migration

- [ ] 3.1 Migrate `LoginView.vue`: replace heading, placeholders, button, error fallbacks, and loading/notification messages with `t()` keys
- [ ] 3.2 Migrate `ChannelsView.vue`: dashboard heading/copy, create button, search placeholder, results-empty message, confirm-dialog title/message, and all notification toasts; ensure `result.message` never surfaces raw per spec (localized fallback wins)
- [ ] 3.3 Migrate `EpisodesView.vue`: back label, refresh button, search placeholder, empty states, and notification messages
- [ ] 3.4 Migrate `HistoryView.vue`: heading, back button aria-label, RSS tooltip, search placeholder, and empty states

## 4. Components migration

- [ ] 4.1 Migrate `AppHeader.vue`: nav links (Channels, History), Logout buttons, drawer user fallback label, aria-labels, and add the language switcher — a radix-vue dropdown in the actions cluster listing autoglottonyms (`English`, `Español`) with the active locale marked, calling `locale.apply(code)`, plus a switch entry inside the mobile drawer
- [ ] 4.2 Migrate `ChannelCard.vue`: tooltips (Open on YouTube, Get RSS feed, Reload cover, Edit channel, Delete channel and audio files), aria-labels, and the dynamic sync-status tooltip using i18n interpolation
- [ ] 4.3 Migrate `EpisodeCard.vue`: play/pause/stop aria-labels, YouTube link, and replace `toLocaleDateString('en-US')` with locale-aware `$d` formatting (spec: locale-aware dates)
- [ ] 4.4 Migrate `Pagination.vue`, `SortControl.vue`, and `SearchInput.vue`: previous/next and sort aria-labels, sort option labels/tooltips, and the default search placeholder
- [ ] 4.5 Migrate `PersistentPlayer.vue`: play/pause/stop/seek/mute/unmute aria-labels
- [ ] 4.6 Migrate `AddChannelDialog.vue` and `ConfirmDialog.vue`: dialog titles, form labels, placeholders, and buttons (edit/new title, Active, Url, max episodes, first episode date, save/change/cancel/create)
- [ ] 4.7 Migrate `AppNotification.vue` and any remaining generic components (`AppButton`, `AppInput`, `AppToggle`, `AppLoading`, `AppDialog`) — remove leftover hardcoded strings, keep internals status-quo where nothing user-facing exists

## 5. Tests and quality gate

- [ ] 5.1 Add a shared mount helper (e.g. `frontend/src/test/i18n.ts`) exporting an `en`-only i18n instance, and wire it into `AppHeader.test.ts` and `HistoryView.test.ts` so existing English assertions keep passing deterministically
- [ ] 5.2 Run `pnpm typecheck`, `pnpm lint`, `pnpm test`, and `pnpm build`; fix any type or lint regressions
- [ ] 5.3 Manual verification: switch to `Español` in a running SPA and confirm headings, dialogs, toasts, dates, empty states, and player aria-labels render in Spanish; confirm choice survives reload and login redirect; confirm unsupported browser language defaults to English