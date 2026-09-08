# playback-progress

## MODIFIED Requirements

### Requirement: Stop halts playback; only the card's stop resets a non-reproducing episode

The player's stop control SHALL halt playback when the current episode is reproducing, flushing its current position so a later resume starts there, and SHALL NEVER reset a saved position when there is no target (persistent-bar stop): on a stopped or paused current episode it SHALL only converge to the stopped state, leaving the saved position untouched. The card's stop control (which passes its episode as target) SHALL: halt a reproducing current episode keeping its position; and reset to 0, keeping the listened mark unchanged, the saved position of the episode it belongs to when that episode is not reproducing — either a non-current card or the current episode when stopped or paused. Internal stops that are not user gestures (end of queue after completion, session teardown) SHALL keep the position as before. From the player itself, the only way to clear a saved position SHALL be the explicit "start over" flow.

#### Scenario: Player-bar stop halts a reproducing episode and keeps its position
- **WHEN** the user presses the persistent bar's stop while an episode is playing at 45 minutes
- **THEN** playback halts and the saved position stays 45 minutes, so the next play resumes there

#### Scenario: Player-bar stop on a stopped or paused episode keeps the position
- **WHEN** the user presses the persistent bar's stop on the current episode that is not reproducing (already stopped, or paused) and has a saved position above 0
- **THEN** the player converges to the stopped state and the saved position is left unchanged

#### Scenario: Card stop on a non-current episode resets that episode
- **WHEN** the user presses a card's stop while another episode is current and the card's episode is not reproducing
- **THEN** that episode's saved position is reset to 0 (listened mark kept) and the current episode's playback is untouched

#### Scenario: Card stop on the current episode when not reproducing resets it
- **WHEN** the user presses the current card's stop while the current episode is stopped or paused
- **THEN** the current episode's saved position is reset to 0, keeping the listened mark

#### Scenario: Card stop on a reproducing current episode halts and keeps the position
- **WHEN** the user presses the current card's stop while the current episode is playing
- **THEN** playback halts and the saved position is kept for a later resume

#### Scenario: Completion keeps the position
- **WHEN** an episode completes and the queue ends, or the session is torn down
- **THEN** the episode halts and its position is kept (no reset)