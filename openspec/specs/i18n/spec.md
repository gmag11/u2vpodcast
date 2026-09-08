## Purpose

The SPA frontend supports multiple display languages selected and remembered by the user. `en` and `es` are shipped; the architecture allows adding locales by adding a message catalogue and a locale entry. All user-facing UI strings come from the translation catalogues — never hardcoded — except content data (YouTube titles/descriptions, RSS metadata), which is not UI text.

## Requirements

### Requirement: Locale detection and default on first visit

On first visit, without a saved preference, the SPA SHALL select the initial locale from the browser's `navigator.language`, matching the browser language against the set of available locales. When the browser language is not supported or detection yields nothing, the SPA SHALL use `en` as the default. The selection SHALL resolve synchronously before the app mounts so no language-flash occurs.

#### Scenario: Browser language maps to a supported locale

- **WHEN** a user with no saved locale preference visits the SPA and their browser language is `es` or `es-ES`
- **THEN** the SPA renders in Spanish

#### Scenario: Unsupported browser language falls back to English

- **WHEN** a user with no saved locale preference visits the SPA and their browser language is `de-DE`
- **THEN** the SPA renders in English

#### Scenario: Locale is set before first paint

- **WHEN** a user visits the SPA with no saved locale preference
- **THEN** the `document.documentElement.lang` attribute reflects the resolved locale before the initial render completes

### Requirement: Locale persistence

The SPA SHALL remember the user's chosen locale in `localStorage` under the `locale` key. The saved locale SHALL be restored on subsequent visits and SHALL survive full page reloads and the login redirect cycle.

#### Scenario: Chosen locale is restored after reload

- **WHEN** a user selects Spanish and then reloads the page
- **THEN** the SPA renders in Spanish without requiring the user to choose again

#### Scenario: Saved locale survives the login flow

- **WHEN** an unauthenticated user selects a locale on the login screen and logs in
- **THEN** the locale selected before login is still active after the redirect to the application

### Requirement: Reactive language switching from the header

The SPA SHALL provide a language selector in the header bar, present on desktop and mobile layouts. The selector SHALL list every available locale by its autoglottonym (`English`, `Español`), SHALL indicate the currently active locale, and selecting an entry SHALL switch the entire UI reactively without a full page reload.

#### Scenario: Switching to Spanish updates the whole UI without reload

- **WHEN** a user opens the language selector, selects `Español`, and stays on the same page
- **THEN** all visible labels, placeholders, notifications triggered afterwards, and dates render in Spanish immediately, and the selector marks `Español` as active

#### Scenario: Selector is reachable on mobile

- **WHEN** a user on a mobile-width screen views a header-rendering page
- **THEN** the language selector is available in the header bar (rendered as an icon-only trigger alongside the theme toggle) and switches locale the same way as on desktop

### Requirement: All user-facing UI strings are localized

Every hardcoded user-facing string in the views and components SHALL be replaced by a translation catalogue lookup. This covers: page headings and body copy, button and link labels, input placeholders, dialog titles and form labels, empty states, tooltips, toast/notification messages, confirm-dialog copy, and the textual portion of header/brand chrome. Text SHALL NOT be concatenated manually from English literals.

#### Scenario: Channels page displays localized headings and actions

- **WHEN** a Spanish-language user opens the channels page
- **THEN** the page heading, search placeholder, create button label, sort control labels, and any empty-state message render in Spanish

#### Scenario: Notifications render localized messages

- **WHEN** a Spanish-language user creates a channel successfully
- **THEN** the resulting toast notification shows the localized Spanish success message

### Requirement: Accessibility attributes are localized

All user-facing `aria-label` and `title` attributes from the translation catalogues SHALL be translated along with their visible counterparts. Screen-reader and tooltip text SHALL switch with the selected locale.

#### Scenario: Player controls expose localized labels to assistive tech

- **WHEN** a Spanish-language user uses the audio player controls
- **THEN** the play/pause/stop/seek/mute buttons expose Spanish `aria-label` values

### Requirement: Locale-aware date formatting

Dates rendered in the UI SHALL be formatted according to the active locale. The SPA SHALL NOT hardcode a fixed locale (such as `en-US`) for date formatting.

#### Scenario: Episode publication date follows the locale

- **WHEN** a Spanish-language user views an episode card
- **THEN** the publication date renders using Spanish date formatting conventions

### Requirement: Fallback to the baseline locale for missing translations

Any translation key missing from the active locale SHALL render using the `en` catalogue instead of showing a raw key. A parity test SHALL fail when a key exists in `en` but is missing from any other shipped locale catalogue.

#### Scenario: Missing Spanish key falls back to English

- **WHEN** a Spanish-language user triggers a string whose key is absent from the `es` catalogue
- **THEN** the English `en` value renders for that string and no raw key is displayed

### Requirement: Backend status strings never surface raw in the UI

API error responses carry HTTP status text (e.g. `"400 Bad Request"`) that is not human-readable UI copy. When an API failure occurs, the SPA SHALL show a localized client-side error message rather than the raw backend status string.

#### Scenario: Failed channel update shows a localized error

- **WHEN** an API update fails and a Spanish-language user is viewing the channels page
- **THEN** the toast shows a Spanish error message derived from the translation catalogue, not the backend status text

### Requirement: Content data is not translated

Channel and episode titles, descriptions, URLs, and other content derived from YouTube or RSS feeds SHALL render exactly as stored, unmodified by the translation layer. The locale catalogue SHALL NOT attempt to translate content data.

#### Scenario: YouTube titles render verbatim in any locale

- **WHEN** a Spanish-language user views an episode list
- **THEN** episode titles and descriptions from YouTube render exactly as stored, while surrounding UI labels render in Spanish