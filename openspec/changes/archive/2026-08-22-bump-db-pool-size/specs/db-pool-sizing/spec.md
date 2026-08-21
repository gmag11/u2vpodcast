## Purpose

Defines the SQLite connection-pool capacity policy: an adequate default, a configuration override, and journal settings that keep concurrent reads healthy.

## ADDED Requirements

### Requirement: SQLite pool has an adequate, configurable connection count

The app SHALL open the SQLite pool with at least 5 connections by default (overridable via the optional `db_pool_max_connections` config key, clamped to >= 1). The pool SHALL be shared process-wide.

#### Scenario: Default pool supports concurrent API + background work
- **WHEN** the app handles a feed request, a channels list, and a background refresh simultaneously
- **THEN** requests are not starved by the 2-connection pool; reads proceed concurrently

#### Scenario: Pool size override takes effect
- **WHEN** `config.yml` sets `db_pool_max_connections: 10`
- **THEN** the pool opens with 10 connections

### Requirement: WAL journal mode and busy timeout are enabled

Pool connections SHALL open the SQLite database in WAL journal mode with a configured busy timeout, so concurrent readers are not blocked by the single writer and transient write conflicts wait instead of failing immediately.

#### Scenario: Concurrent read and write do not surface SQLITE_BUSY
- **WHEN** a refresh writes new episodes while API reads are in flight
- **THEN** reads continue and the write completes within the busy timeout (no `database is locked` error) under the configured pool size