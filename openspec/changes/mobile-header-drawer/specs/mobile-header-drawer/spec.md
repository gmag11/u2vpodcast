## ADDED Requirements

### Requirement: Mobile header shows the brand icon only

On screens narrower than the `md` breakpoint, the header brand SHALL render only the brand icon; the "U2V Podcast" wordmark SHALL be hidden. On `md+` screens the full brand (icon + wordmark) SHALL render as today.

#### Scenario: Mobile shows icon-only branding
- **WHEN** an authenticated user views a header-rendering page on a mobile-width screen
- **THEN** the header shows only the brand icon, with no wordmark text

#### Scenario: Desktop keeps the full brand
- **WHEN** an authenticated user views a header-rendering page on an `md+` screen
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

### Requirement: Mobile search expands from a toggle

On views that provide a header search slot (the Channels page), the header SHALL render a search toggle on screens narrower than `md`. Tapping it SHALL expand a full-width search input row directly below the header bar; tapping it again (or closing the row) SHALL collapse it. On `md+` screens the search input SHALL render inline in the header bar as today.

#### Scenario: Mobile search expands below the header
- **WHEN** a user on a mobile-width screen taps the search toggle on the Channels page
- **THEN** a full-width search input appears below the header and filters the channel list as the user types

#### Scenario: Mobile search collapses again
- **WHEN** a user taps the search toggle while the mobile search row is expanded
- **THEN** the search row collapses

#### Scenario: Desktop search stays inline
- **WHEN** a user views the Channels page on an `md+` screen
- **THEN** the search input renders inline in the header bar, as today
