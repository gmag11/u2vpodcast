## ADDED Requirements

### Requirement: Player reopen control

When the persistent player bar is hidden (after its auto-hide delay or because no episode is loaded), the app SHALL render a small floating control that reveals the bar without starting playback. The control SHALL be visible only while the bar itself is hidden. Activating it SHALL restore the persistent player bar to its visible state, preserving the current episode, the up-next queue, and the stopped/paused state, and SHALL NOT call play on any episode. The control SHALL be available in both the wide (desktop) and compact (mobile) compositions. If playback is stopped, the bar SHALL resume its normal auto-hide behavior after being reopened, exactly as if it had become visible through any other path.

#### Scenario: Bar is hidden and the reopen control appears
- **WHEN** the persistent player bar becomes hidden after a stop and its auto-hide delay elapses
- **THEN** a floating reopen control is rendered on screen

#### Scenario: Reopening the bar does not start playback
- **WHEN** the user activates the reopen control while the player is stopped with a current episode and queue preserved
- **THEN** the persistent bar is shown again, the current episode and queue remain intact, and no episode starts playing

#### Scenario: Reopen control hides once the bar is visible
- **WHEN** the user activates the reopen control or playback starts through any other path
- **THEN** the reopen control disappears because the bar is visible again

#### Scenario: Reopened stopped bar still auto-hides
- **WHEN** the player is stopped, the user reopens the bar via the control, and no new playback starts
- **THEN** the bar remains visible for 10 seconds and then hides again with the downward slide animation, while the reopen control reappears

#### Scenario: No reopen control while the bar is visible or nothing to restore
- **WHEN** the bar is currently visible, or there is no current episode and no up-next queue to restore
- **THEN** no reopen control is rendered
