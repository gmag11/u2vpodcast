## ADDED Requirements

### Requirement: Global spacebar toggles active playback

While the application document has focus, the shared player SHALL treat a spacebar key press as a global play/pause toggle when an episode is playing or paused. The shortcut SHALL pause playing audio and resume paused audio. It SHALL have no effect when the player is stopped or no episode is loaded. The shortcut SHALL NOT override spacebar behavior while focus is in an editable field or an interactive control with native spacebar behavior.

#### Scenario: Spacebar pauses playing audio
- **WHEN** an episode is playing, the application document has focus, and the user presses the spacebar outside an editable field or interactive control
- **THEN** the shared player pauses the episode and prevents the page's default spacebar action

#### Scenario: Spacebar resumes paused audio
- **WHEN** an episode is paused, the application document has focus, and the user presses the spacebar outside an editable field or interactive control
- **THEN** the shared player resumes the episode and prevents the page's default spacebar action

#### Scenario: Spacebar has no effect after stop
- **WHEN** the player is stopped and the user presses the spacebar
- **THEN** playback remains stopped and the key press does not start or resume an episode

#### Scenario: Spacebar has no effect without a loaded episode
- **WHEN** no episode is loaded and the user presses the spacebar
- **THEN** no playback starts and player state remains unchanged

#### Scenario: Focused controls retain spacebar behavior
- **WHEN** focus is in an editable field or an interactive control with native spacebar behavior and the user presses the spacebar
- **THEN** the shared player state remains unchanged and the focused element retains its normal spacebar behavior

#### Scenario: Unfocused document ignores spacebar
- **WHEN** the application document does not have focus and a spacebar key event occurs
- **THEN** the shared player state remains unchanged
