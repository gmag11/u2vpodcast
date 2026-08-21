## Purpose

Guarantees that the app never crashes through recursion in conversion impls, a misconfigured session key, or missing error metadata. Configuration mistakes fail fast with clear messages; transient runtime failures degrade to error responses instead of panics.

## ADDED Requirements

### Requirement: Recursive conversion impls are not reachable

There SHALL be no `From` implementation whose body calls `into()`/`from()` in a way that resolves to itself (infinite recursion). Channel, episode and response conversion paths SHALL produce a value or a compile error — never a stack overflow.

#### Scenario: Converting a channel to a Value does not hang
- **WHEN** a `Channel` (or `Episode`) is converted to a `serde_json::Value`
- **THEN** the conversion completes without recursing, returning the JSON representation

#### Scenario: CustomResponse conversion does not recurse
- **WHEN** a `CustomResponse<T>` is converted into an `HttpResponse`
- **THEN** a valid HTTP response is produced without infinite recursion

### Requirement: Startup validates the session secret key length

At startup, in both production and development mode, the configured `secret_key` SHALL be checked to be at least 64 bytes after trimming. An undersized key SHALL abort startup with a clear, actionable error message instead of panicking inside the session middleware.

#### Scenario: Short secret key aborts startup with a clear error
- **WHEN** `config.yml` defines a `secret_key` shorter than 64 bytes and the app starts
- **THEN** startup fails with a message explaining the minimum length and how to generate a valid key

#### Scenario: Valid secret key starts normally
- **WHEN** `config.yml` defines a `secret_key` of 64+ bytes and the app starts
- **THEN** startup proceeds and the session middleware is built

### Requirement: Serializing an Error never panics

The `Serialize` impl of `Error` SHALL produce a numeric `status_code` for any `Error` instance, including those created via `Error::default` (no explicit status). The value SHALL match the effective HTTP status used by the `ResponseError` impl (500 fallback).

#### Scenario: Error with no explicit status serializes as 500
- **WHEN** an `Error::default("...")` is serialized
- **THEN** serialization succeeds and the `status_code` field is `500`

### Requirement: Session write failures fail the request, not the process

Handlers that write claims to the session SHALL handle write failures by returning an error response (e.g. HTTP 500) and logging the failure; they SHALL NOT panic the request handler.

#### Scenario: Session insert failure returns 500
- **WHEN** inserting a session claim fails in the login handler
- **THEN** the handler returns an HTTP 500 response and the process keeps serving