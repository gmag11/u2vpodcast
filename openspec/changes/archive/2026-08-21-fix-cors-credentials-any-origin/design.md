## Context

Production CORS branch in `src/main.rs` builds an allowlist and then calls `allow_any_origin()`, which in actix-cors 0.7 resets the list to `All`. With `supports_credentials()` the middleware reflects any request origin back. The cookie already ships cross-site (`SameSite=None; Secure`), so the combination is a full CSRF + read-exfiltration primitive.

## Goals / Non-Goals

**Goals:**
- Production CORS allows only explicitly configured origins with credentials.
- Startup fails fast on an invalid configured origin.

**Non-Goals:**
- No change to the development CORS branch.
- No change to cookie attributes in this change (assessed separately if needed).

## Decisions

- **Delete `allow_any_origin()` from the production branch and keep explicit origins.** The dev branch keeps any-origin, which is the correct dev posture.
- **Normalize origin strings:** strip trailing slashes; express the configured `url` as `scheme://host[:port]`. The `https://yt3.googleusercontent.com` entries are origin-normalized as well (no trailing slash) so the frontend cover-image fetch keeps working — that host only serves unauthenticated images, so it stays in the allowlist without credentials concerns.
- **Validate at startup and fail fast.** A wrong origin silently deployed is worse than a loud refusal; parse each production allowlist entry with `url::Url` and require scheme + host.

## Risks / Trade-offs

- [Tightening CORS may block a legitimately configured origin if the operator typoes it] → Mitigated by fail-fast validation and a clear error naming the invalid entry.
- [Multiple deployment origins require config plumbing] → Accepted; a hardcoded secondary image host plus the primary configured origin covers current deployments.

## Migration Plan

1. Change CORS builder + origin validation in `src/main.rs`.
2. Update `config.yml` example/sample files to use full origins.
3. Deploy and verify preflight (`OPTIONS`) and simple requests for both allowed and disallowed `Origin` values.

## Open Questions

None.