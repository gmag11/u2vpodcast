# api-response-contract

## Purpose

Defines that each endpoint honors its declared pagination contract and that error responses report their real HTTP status while preserving the response body schema.

## Requirements

### Requirement: The channels list honors pagination

`GET /api/1.0/channels/` SHALL accept a `page` query parameter and return only that page of channels (page size from `config.per_page`), following the same convention as the users and episodes endpoints. Consecutive pages SHALL contain disjoint channel sets.

#### Scenario: Second page returns the next subset
- **WHEN** a deployment has more channels than `per_page` and `?page=2` is requested
- **THEN** the response contains the next subset of channels and no channel appears on both page 1 and page 2

#### Scenario: Default page is the first
- **WHEN** no `page` parameter is provided
- **THEN** the first page is returned

### Requirement: Error responses carry the true HTTP status

All error responses, including those built through `CResponse::ko`, SHALL use the failing status code in the HTTP status line (e.g. `401` for login failure, `400` for bad request). The response body SHALL keep its `status`, `status_code`, `message`, `user`, `data` shape, with `status_code` consistent with the HTTP status.

#### Scenario: Bad login returns HTTP 401
- **WHEN** login is attempted with wrong credentials
- **THEN** the HTTP response status is `401` and the body `status_code` is `401`, `status` is `false`

#### Scenario: Success responses unchanged
- **WHEN** a handler returns success through `CResponse::ok`
- **THEN** the HTTP status remains `200` with the existing envelope