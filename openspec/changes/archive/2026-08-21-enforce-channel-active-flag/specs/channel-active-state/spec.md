## ADDED Requirements

### Requirement: Scheduled sync processes only active channels

The periodic sync worker SHALL iterate only channels whose `active` flag is true. A channel with `active = false` SHALL NOT be refreshed, fetched, or cleaned by the scheduled worker in a cycle, while its data remains stored and its status values stay untouched by that cycle. Activating or deactivating a channel SHALL take effect on the next sync cycle.

#### Scenario: Inactive channel is skipped by the worker
- **WHEN** a channel has `active = false` and the scheduled worker runs
- **THEN** the worker does not download or process that channel, and its `last_sync_at`/`last_sync_ok` values are not modified by that run

#### Scenario: Active channel is processed as before
- **WHEN** a channel has `active = true` and the scheduled worker runs
- **THEN** the channel is updated and cleaned exactly as before this change

#### Scenario: Toggle takes effect on the next cycle
- **WHEN** an operator deactivates a channel through the UI and the next scheduled cycle runs
- **THEN** the channel is skipped from that cycle onward