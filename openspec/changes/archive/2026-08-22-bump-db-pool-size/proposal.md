## Why

The sqlite pool is fixed at `max_connections(2)` (`src/main.rs:160`). The app runs two actix workers that serve concurrent API requests (channels, episodes, users, feed generation) plus a background worker task plus fire-and-forget refresh tasks — all sharing those two connections. Any long query (feed generation joining all episodes/channels, paginated reads during a refresh burst) holds one of the two slots; the second slot being busy leaves further requests queued behind the pool, causing latency spikes for unrelated endpoints. SQLite handles concurrent *reads* well once the journal is in WAL mode; the only sequential point is writes.

## What Changes

- Raise the default connection count from 2 to a value that comfortably fits workers + background + a couple of in-flight API requests (default 5).
- Enable WAL journal mode and a busy timeout on the pool connections so concurrent readers and the rare writer do not block each other with `SQLITE_BUSY`.
- Make the pool size overridable through a new optional `config.yml` key (`db_pool_max_connections`) with the raised default applied when absent.

## Capabilities

### New Capabilities

- `db-pool-sizing`: Defines the SQLite pool capacity policy (defaults, configuration override, journal mode).

### Modified Capabilities

(none)

## Impact

- `src/main.rs` (pool creation), `src/models/config.rs` (new optional key + default).
- No API contract change; no schema change.
- Minor memory footprint per extra connection (SQLite is lightweight).

## Non-Goals

- No migration to another database.
- No change to transactional/business logic.
- No unbounded pool size; the configurable cap is intentional.