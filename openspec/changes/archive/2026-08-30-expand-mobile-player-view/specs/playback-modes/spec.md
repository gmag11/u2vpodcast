## ADDED Requirements

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
