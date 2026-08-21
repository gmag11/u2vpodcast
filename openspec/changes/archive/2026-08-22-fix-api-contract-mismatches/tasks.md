## 1. Real pagination for the channels endpoint

- [x] 1.1 In `src/handlers/channels.rs`, make `read_with_pagination` accept `Query<Page>` (as in `users.rs`) and call `Channel::read_with_pagination(&data.pool, page, per_page)` with `per_page` from `config.per_page`
- [x] 1.2 Remove the `#[allow(unused)]` on `Channel::read_with_pagination` once wired
- [x] 1.3 Keep the `CResponse::ok` envelope; verify the route returns page-subsets

## 2. Real HTTP status on error responses

- [x] 2.1 Change `CResponse::ko` (`src/models/response.rs`) to build the response with the passed `status_code` instead of `StatusCode::OK`
- [x] 2.2 Confirm the body fields (`status:false`, `status_code`, `message`, `user`) are unchanged
- [x] 2.3 Verify the `Error`/`ResponseError` path already returns the real status and stays untouched

## 3. Frontend & contract verification

- [x] 3.1 Grep the SPA for login/logout error handling and channel-list rendering; adapt any code assuming HTTP 200 on error or a full unfiltered channel list
- [x] 3.2 Update `sample.http`/`sample.hurl` assertions that expected HTTP 200 on error paths
- [x] 3.3 Add tests: `CResponse::ko` yields the given status line; channels page 1/page 2 do not overlap

## 4. Verification

- [x] 4.1 `cargo test` passes
- [x] 4.2 Manual: bad login returns HTTP 401; `GET /channels/?page=2` returns the second page subset
- [x] 4.3 Full SPA smoke test (login, channel list, episodes)