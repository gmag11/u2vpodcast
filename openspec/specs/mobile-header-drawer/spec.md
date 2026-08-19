## Purpose

Reworks the SPA header to be mobile-friendly: icon-only branding on narrow screens, navigation moved into a right-edge side drawer below `md`, per-view actions collapsing to icon-only buttons on mobile, a persistent theme toggle, and search relocated from the header into page content.

## Requirements

### Requirement: Mobile header shows the brand icon only

On screens narrower than the `lg` breakpoint, the header brand SHALL render only the brand icon; the "U2V Podcast" wordmark SHALL be hidden. On `lg+` screens the full brand (icon + wordmark) SHALL render. The wordmark is hidden between `md` and `lg` as well so the header fits narrow desktop widths.

#### Scenario: Mobile shows icon-only branding
- **WHEN** an authenticated user views a header-rendering page on a mobile-width screen
- **THEN** the header shows only the brand icon, with no wordmark text

#### Scenario: Desktop keeps the full brand
- **WHEN** an authenticated user views a header-rendering page on an `lg+` screen
- **THEN** the header shows the brand icon and the "U2V Podcast" wordmark

### Requirement: Mobile navigation moves into a side drawer

On screens narrower than `md`, the inline nav links (Channels, History) SHALL NOT render in the header bar. The header SHALL instead show a hamburger button that opens a drawer from the right edge of the screen. The drawer SHALL contain the logged-in user (avatar and name), the Channels and History nav links, and a Logout control. Selecting a nav link or Logout, clicking the backdrop, pressing Escape, or navigating to another route SHALL close the drawer.

#### Scenario: Hamburger opens the navigation drawer
- **WHEN** a user on a mobile-width screen taps the hamburger button
- **THEN** a drawer slides in from the right with the user's avatar and name, the Channels and History links, and a Logout control, and a backdrop covers the page

#### Scenario: Nav links are hidden from the mobile bar
- **WHEN** a user views a header-rendering page on a mobile-width screen
- **THEN** the Channels and History links are not rendered in the header bar itself

#### Scenario: Selecting a nav link closes the drawer
- **WHEN** a user selects the History link inside the open drawer
- **THEN** the drawer closes and the router navigates to the History screen

#### Scenario: Backdrop click closes the drawer
- **WHEN** the drawer is open and the user clicks the backdrop
- **THEN** the drawer closes

#### Scenario: Escape closes the drawer
- **WHEN** the drawer is open and the user presses Escape
- **THEN** the drawer closes

#### Scenario: Desktop renders the inline nav
- **WHEN** a user views a header-rendering page on an `md+` screen
- **THEN** the Channels and History links render inline in the header bar and no hamburger button is shown

### Requirement: Per-view header actions render as icon-only buttons on mobile

On screens narrower than `md`, per-view header actions (Create New on Channels, Refresh on Episodes) SHALL render as icon-only buttons with their text hidden. On `md+` screens the actions SHALL render with their current text-and-icon layout.

#### Scenario: Mobile action is icon-only
- **WHEN** a user on a mobile-width screen views the Channels page
- **THEN** the Create New action renders as an icon-only button without visible text, and still opens the new-channel dialog when tapped

#### Scenario: Desktop action keeps its text
- **WHEN** a user views the Channels page on an `md+` screen
- **THEN** the Create New action renders with its text label and icon, as today

### Requirement: Theme toggle stays visible on mobile

The theme toggle SHALL remain visible in the header bar on mobile-width screens, identical to the desktop behavior.

#### Scenario: Theme toggle available on mobile
- **WHEN** a user on a mobile-width screen views a header-rendering page
- **THEN** the theme toggle is visible in the header bar and toggles light/dark as on desktop

### Requirement: Search lives in page content, not the header

The header SHALL NOT render a search input or search toggle. Search inputs SHALL be placed in the page content above the list they filter, matching the episodes list pattern. On the Channels page, a search input SHALL render above the channel cards at all widths (mobile and desktop), and typing in it SHALL filter the channel list live.

#### Scenario: Channels search sits above the cards
- **WHEN** an authenticated user opens the Channels page on mobile or desktop
- **THEN** a search input renders above the channel cards and filters the list as the user types
