## Purpose

Defines playback modes — shuffle, repeat-all, repeat-one — applied to the up-next queue.

## Requirements

### Requirement: Shuffle mode over the up-next queue

When shuffle is enabled, the player SHALL consume the up-next queue in a randomized order derived from the seeded order; the underlying playlist/list order SHALL remain unchanged. Disabling shuffle SHALL restore the authored order. Each shuffle cycle (including repeat-all turnovers) SHALL use a fresh randomization.

#### Scenario: Enabling shuffle randomizes play order
- **WHEN** the user enables shuffle and plays through the queue
- **THEN** episodes play in a randomized order that still contains exactly the queued set

#### Scenario: Disabling shuffle restores order
- **WHEN** the user disables shuffle mid-queue
- **THEN** the remaining episodes play in the original authored order

#### Scenario: Queue set is preserved under shuffle
- **WHEN** shuffle is active
- **THEN** no episode plays twice and none is skipped until the queue is exhausted

### Requirement: Repeat modes

The player SHALL support repeat-none (default), repeat-all, and repeat-one. With repeat-all, when the queue is exhausted the player SHALL rebuild the queue from its original seed (re-shuffled if shuffle is active) and continue. With repeat-one, the finished episode SHALL replay from the start.

#### Scenario: Repeat-all restarts the queue
- **WHEN** repeat-all is active and the last queued episode finishes
- **THEN** the queue is rebuilt from the original playlist/order and the next episode starts

#### Scenario: Repeat-one replays the episode
- **WHEN** repeat-one is active and an episode finishes
- **THEN** the same episode starts over from the beginning

#### Scenario: Repeat-none stops at the end
- **WHEN** repeat is none and the last queued episode finishes
- **THEN** the player stops and the queue is cleared

### Requirement: Mode controls in the persistent bar

The persistent player bar SHALL expose shuffle and repeat toggles reflecting the player's mode state. The repeat control SHALL cycle through none, all, and one; active modes SHALL be visually indicated.

#### Scenario: Cycling the repeat mode
- **WHEN** the user presses the repeat control repeatedly
- **THEN** the mode cycles none → all → one → none and each state is visually indicated in the bar

#### Scenario: Toggling shuffle
- **WHEN** the user presses the shuffle control
- **THEN** shuffle toggles between on and off and the on state is visually indicated

### Requirement: Combined shuffle/repeat control in the mobile expanded view

The expanded now-playing view (specified by the `persistent-audio-player` capability) SHALL expose a single combined control that cycles through three mutually exclusive visual states: normal order, repeat, and shuffle. Pressing the control SHALL advance it to the next state in that order, wrapping from shuffle back to normal order.

Selecting "normal order" SHALL set shuffle off and repeat to none. Selecting "repeat" SHALL set shuffle off and repeat to all. Selecting "shuffle" SHALL set shuffle on and repeat to none. The control SHALL reflect the current combination of the store's shuffle and repeat state when it matches one of these three states; if the store holds a combination outside these three (for example repeat-one, or shuffle combined with a repeat mode), the control SHALL display the closest represented state without changing the underlying store state until the user interacts with it.

This combined control is additional to, and does not replace, the independent shuffle toggle and none/all/one repeat cycle already exposed by the wide composition of the persistent bar.

#### Scenario: Cycling from normal order to repeat
- **WHEN** the combined control shows "normal order" and the user presses it
- **THEN** the control shows "repeat", shuffle is set off, and repeat is set to all

#### Scenario: Cycling from repeat to shuffle
- **WHEN** the combined control shows "repeat" and the user presses it
- **THEN** the control shows "shuffle", shuffle is set on, and repeat is set to none

#### Scenario: Cycling from shuffle back to normal order
- **WHEN** the combined control shows "shuffle" and the user presses it
- **THEN** the control shows "normal order", shuffle is set off, and repeat is set to none

#### Scenario: Control reflects a state set from the wide composition
- **WHEN** the user enables shuffle and sets repeat to all from the wide composition, then opens the mobile expanded view
- **THEN** the combined control shows the state closest to that combination without altering shuffle or repeat until the user presses it

#### Scenario: Repeat-one is not directly reachable from the combined control
- **WHEN** the combined control is used exclusively (no interaction with the wide composition)
- **THEN** repeat-one is never selected, since the combined control only cycles through normal order, repeat (all), and shuffle

### Requirement: Modes persist across reloads

Shuffle and repeat mode SHALL be persisted in the browser and restored on app load.

#### Scenario: Reload keeps the modes
- **WHEN** the user enables shuffle with repeat-all and reloads the page
- **THEN** both modes are still active after the reload
