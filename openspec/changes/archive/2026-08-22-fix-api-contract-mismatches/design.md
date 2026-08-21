## Context

**Pagination:** `Channel::read_with_pagination(pool, page, per_page)` exists (`src/models/channel.rs:295`, `#[allow(unused)]`). The handler named `read_with_pagination` does not use it — it calls `read_all`. The other two paginated resources (users, episodes) take `page` from a `Query<Page>` struct (`page: Option<i64>`, unwrap_or(1)) and use `config.per_page`. Channels should mirror them. Pagination sorting for channels `ORDER BY created_at ASC LIMIT $1 OFFSET $2` differs from the channels list ordering capability (`channels-list-ordering`), but the model method already exists; the handler just needs to match the users/episodes convention. (Whether the list should order by last_date DESC is a separate spec concern; keep the model's `created_at ASC` — consistent with the existing paginated resources.)

**Error status:** `CResponse::ko(status_code, session)`:
```rust
HttpResponse::build(StatusCode::OK).json(response)
```
always 200. `post_login` returns `CResponse::ko(Status::UNAUTHORIZED, ...)` on bad credentials; `get_logout` uses `BAD_REQUEST`. Meanwhile the `Error`/`ResponseError` path already returns the real status. Inconsistent: same logical failure, different HTTP status depending on which helper produced it.

## Goals / Non-Goals

**Goals:**
- `/channels/` honors page/per_page like users and episodes.
- Error responses (including `CResponse::ko`) carry the true HTTP status; body shape unchanged.

**Non-Goals:**
- No body-schema change (`status`/`status_code`/`message`/`user`/`data`).
- No pagination metadata object; page size defaults to `config.per_page` exactly like others.
- No change to `CResponse::ok` or `purge`.

## Decisions

- **Channel pagination:** change the handler to accept `Query<Page>` (same struct as users) and call `Channel::read_with_pagination(&data.pool, page, per_page)`. Defaults and types identical to `users.rs`. Keep the `CResponse::ok` envelope. Note the route stays `GET /channels/`; the page query param is addressed the same way `GET /users/?page=N` works.
  - **Ordering decision (deviation from initial draft):** the model method now paginates with the **same ORDER BY as `read_all`** (`last_date IS NULL, last_date DESC` — most recent activity first) instead of `created_at ASC`. Rationale: the SPA channel dashboard renders the `read_all` ordering (and `channels-list-ordering` governs sorting); paginating with a different order would silently reorder the UI between page transitions. Pages remain disjoint and bounded.
- **`CResponse::ko` HTTP status:** replace `StatusCode::OK` with `status_code` in `HttpResponse::build(...)`. Body building is unchanged, so `status_code` field and body `status:false` remain set. This makes login-failure return 401 at the HTTP layer.
- **Frontend compat:** grep the SPA for `CResponse::ko` consumers and login/logout error handling; if code treats non-200 as transport failure, adapt it to read the body `status_code` (or simply treat 401 as the login-failure it is). Change notes cite this.
- **Error::status_code default:** keep 500 fallback; unaffected.

## Risks / Trade-offs

- [Changing channels/ to return a subset may break SPA "show all" screens] → The SPA already pages episodes/users; apply the same page control to its channel list screen. Any screen that relied on the full list without pagination must be updated — this is the point of the change.
- [Login failure now 401 at HTTP level may break reverse proxies/caches] → Desired semantics; a proxy that treats 401 as "not logged in" behaves correctly for this API.
- [Body `status_code` and HTTP status now always agree for `ko`] → That is the fix.

## Migration Plan

1. Wire `Query<Page>` + `Channel::read_with_pagination` in the channels handler.
2. Change `CResponse::ko` to build with the real status.
3. Unit test `CResponse::ko` (status line) and a channels pagination test (second page excludes first page rows).
4. SPA pass: login failure path and channel list pagination.
5. If any automated `sample.http`/`sample.hurl` assertions assumed HTTP 200 on error, update them.

## Open Questions

- Should `GET /channels/` also accept an explicit `per_page` query param like the model method allows, or keep it config-only like users/episodes? (Default: match users/episodes — `page` only, `per_page` from config.)