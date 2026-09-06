## ADDED Requirements

### Requirement: Player reopen control

When the persistent player bar is hidden (because playback is stopped and its auto-hide delay elapsed, or because there is nothing to restore), the app SHALL render a small floating control that reveals the bar without starting playback. The control SHALL be visible only while the bar itself is hidden and there is restorable content (a current episode or a non-empty up-next queue). Activating it SHALL restore the persistent player bar to its visible state, preserving the current episode, the up-next queue, and the stopped/paused state, and SHALL NOT call play on any episode. The control SHALL be available in both the wide (desktop) and compact (mobile) compositions. If playback is stopped, the bar SHALL resume its normal auto-hide behavior after being reopened, exactly as if it had become visible through any other path. On a page load with a restored stopped queue but no current episode, the bar SHALL be hidden and the reopen control SHALL be the available path back to the bar without starting playback.

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

#### Scenario: Queue-only bar is hidden and restorable via the control
- **WHEN** the player is stopped with no current episode but a non-empty up-next queue (a restored queue), and the auto-hide delay elapses or the page loads in that state
- **THEN** the bar is hidden and a floating reopen control is rendered; activating it shows the bar with the queue intact and no episode starts playing

## MODIFIED Requirements

### Requirement: Animated auto-hide on stop

When playback stops (user presses stop, or the audio reaches its end), the persistent bar SHALL remain visible for 10 seconds and then disappear with a downward slide animation, regardless of whether the up-next queue still holds episodes. Any new play action SHALL bring the bar back with an upward slide animation. While audio is playing or paused mid-track (not stopped), the bar SHALL remain visible. When the player is stopped with no current episode and a non-empty up-next queue (a restored queue without a loaded episode), the bar SHALL auto-hide on the same 10-second delay instead of staying visible. While the bar is hidden and the player holds a current episode or a non-empty queue, the reopen control SHALL offer the way to bring the bar back without starting playback.

#### Scenario: Bar hides after stop with delay and animation
- **WHEN** the user presses stop and no new playback starts
- **THEN** the bar stays visible for 10 seconds, then animates downward and is removed from view

#### Scenario: Play resumes before the hide delay
- **WHEN** the user presses play again within the 10-second delay after stop
- **THEN** the bar stays visible and playback resumes without disappearing

#### Scenario: Paused mid-track keeps the bar visible
- **WHEN** the user pauses an episode without stopping it
- **THEN** the bar remains visible and shows the paused state; it does not auto-hide

#### Scenario: Bar auto-hides even with queued episodes
- **WHEN** the user stops playback while the up-next queue still holds episodes
- **THEN** the bar starts its 10-second auto-hide delay like any other stop

#### Scenario: Queue-only bar auto-hides on the same delay
- **WHEN** the player is stopped with no current episode but a non-empty up-next queue, and no new playback starts
- **THEN** the bar stays visible for 10 seconds and then auto-hides, and the reopen control appears so the queue stays reachable without starting playback
