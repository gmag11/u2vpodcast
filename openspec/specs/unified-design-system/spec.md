## Purpose

Defines the single visual design system for the Vue 3 SPA. The source of truth is the four high-fidelity HTML screens in the Stitch project "Modern Podcast Dashboard Redesign"; the system unifies them into one coherent set of tokens: a coral/sky palette built on the project brand colors, light and dark themes with a manual toggle, one font pairing (Geist + Inter), one icon set (Phosphor), and responsive layouts.

## Requirements

### Requirement: Design source of truth is the Stitch HTML screens

The Vue SPA SHALL be implemented following the design captured in the Stitch project "Modern Podcast Dashboard Redesign" (project `projects/1098547267709376206`). The Stitch MCP server SHALL be used as a reference during implementation. The four HTML screens are the authoritative source for layout, spacing, and component structure:

- **Login**: screen `9dba5a0491e444a7a718cee5a72d7db0` ("Modern Login Screen - HTML High Fidelity") — radial-gradient background, glass-card centered form.
- **Channels**: screen `90b4d6eb80be4fd78028a4897ef73140` ("Modern Podcast Dashboard - HTML Version") — fixed top nav, search, create button, glass-card channel grid.
- **Chapters**: screen `95dd5ddccd2742ceae1042cc607a2eb1` — episode list with search bar and per-episode player controls.
- **Add dialog**: screen `d2b08930a0954c49b569317273e7e13c` ("New Channel Modal - HTML Code") — modal with active toggle, URL, max episodes, first episode date, and create/cancel actions.

#### Scenario: Implementer needs the authoritative screen markup
- **WHEN** an implementer works on a screen and needs its exact structure
- **THEN** they SHALL read the corresponding Stitch HTML screen via the Stitch MCP tools (e.g., `get_screen`) and use it as the layout reference

#### Scenario: Placeholder images must not be copied
- **WHEN** the Stitch HTML screens contain example images (channel covers, episode thumbnails, user avatar) served from `aida-public` URLs
- **THEN** the implementer SHALL NOT copy those images into the SPA; only CSS backgrounds and icons may be reused, and content images SHALL be bound to real `channel.image` / `episode.image` data

### Requirement: Unified design system across all screens

The SPA SHALL implement a single coherent design system shared by all four screens (Login, Channels, Chapters, Add dialog), unifying the distinct styles found in the four Stitch HTML references. Screens SHALL use one palette, one font pairing, one icon set, and one set of UI primitives.

#### Scenario: All screens share the same tokens
- **WHEN** a user navigates between Login, Channels, Chapters, and the Add dialog
- **THEN** every screen renders with the same color tokens, typography scale, border radius, and spacing defined by the shared design system

#### Scenario: No per-screen palette divergence
- **WHEN** two screens are displayed side by side
- **THEN** they SHALL NOT show mutually incompatible colors, fonts, or icon families

### Requirement: Theme color palette

The design system SHALL be based on the existing project brand colors instead of the purple/indigo palettes in the Stitch HTML references. Primary color SHALL be coral `#fe795d` (with the 50–900 scale already defined in the current app), accent SHALL be sky blue `#0ea5e9` (sky-400/sky-500 family used in the current app), and semantic colors SHALL be red for errors/deletion, green for success, and gray/slate for neutral surfaces.

#### Scenario: Primary buttons use coral
- **WHEN** a primary action button is rendered (e.g., "Create channel", "Login")
- **THEN** its background is a coral `#fe795d`-based token

#### Scenario: Links and secondary accents use sky
- **WHEN** a link or secondary accent element is rendered (e.g., "YouTube" link, player highlight)
- **THEN** it uses a sky `#0ea5e9`-based token

### Requirement: Light and dark themes with manual toggle

The design system SHALL provide both a light and a dark theme defined as token sets. The application SHALL render a theme toggle control in the app header. The selected theme SHALL persist in `localStorage`, and on first visit the initial theme SHALL be resolved from `prefers-color-scheme`, falling back to dark when the media query is unavailable.

#### Scenario: Toggle switches theme
- **WHEN** the user clicks the theme toggle while the app is in dark mode
- **THEN** the app switches to light mode, all tokens update, and the choice is saved in `localStorage`

#### Scenario: Theme persists across reloads
- **WHEN** the user reloads the page after having selected light mode
- **THEN** the app renders in light mode without flashing the previous theme

#### Scenario: First visit follows system preference
- **WHEN** a user with no saved preference visits the app on a system configured to dark mode
- **THEN** the app renders in dark mode

### Requirement: Typography and icons are unified

All screens SHALL use a single font pairing: Geist for headings/labels and Inter for body text. All icons SHALL come from a single icon set (Phosphor), replacing the mixed Material Symbols / Phosphor / inline SVG usage found in the Stitch HTML references.

#### Scenario: Consistent font rendering
- **WHEN** text renders on any screen
- **THEN** headings use Geist and body text uses Inter

#### Scenario: Consistent icon rendering
- **WHEN** an icon renders on any screen (search, play, edit, delete, link, close, logout, theme)
- **THEN** it comes from the Phosphor icon set

### Requirement: Responsive layout

Every screen SHALL be responsive. Layouts SHALL reflow from the desktop composition in the Stitch references to smaller viewports: the channel grid SHALL collapse to fewer columns (3 → 2 → 1), the top nav search SHALL hide on small screens, the episode card thumbnail+info row SHALL stack vertically on narrow viewports, and the login/add-dialog cards SHALL stay centered and fill available width without overflow.

#### Scenario: Channel grid on a narrow viewport
- **WHEN** the Channels page renders on a viewport narrower than a tablet
- **THEN** channel cards stack in a single column instead of the three-column desktop grid

#### Scenario: Episode card on a narrow viewport
- **WHEN** the Chapters page renders on a narrow viewport
- **THEN** the episode thumbnail and its text info stack vertically, and player controls remain fully visible
