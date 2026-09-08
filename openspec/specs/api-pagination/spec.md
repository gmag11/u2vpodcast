# api-pagination

## Purpose

Defines how the JSON API paginates list endpoints (users, channels, episodes). A requested page lower than 1 is clamped to page 1 so malformed parameters never produce an invalid SQL OFFSET or a 500, while valid pages behave exactly as before.

## Requirements

### Requirement: Pagination clamps the page number to a valid minimum

Paginated endpoints SHALL treat a requested page lower than 1 as page 1 rather than producing an invalid SQL OFFSET or an error response. Valid pages (`>= 1`) SHALL yield exactly the same results as before the change.

#### Scenario: Page zero behaves as page one
- **WHEN** a client requests `page=0`
- **THEN** the endpoint responds `200` with the first page's items

#### Scenario: Negative page behaves as page one
- **WHEN** a client requests `page=-3`
- **THEN** the endpoint responds `200` with the first page's items

#### Scenario: Valid page is unchanged
- **WHEN** a client requests `page=2`
- **THEN** the endpoint returns exactly the same second-page items as before the change

### Requirement: Consistency across paginated endpoints

The same clamp SHALL apply to every paginated endpoint (users, channels, episodes) so none of them can return a 500 from a malformed page parameter.

#### Scenario: All paginated endpoints share the clamp
- **WHEN** a client sends `page=0` to the users, channels, and episodes paginated endpoints
- **THEN** each responds `200` with its first page instead of a 500
