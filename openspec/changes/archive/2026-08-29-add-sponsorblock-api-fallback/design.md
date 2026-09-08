## Context

The SponsorBlock client currently owns one base URL and performs blocking HTTP retrieval in an Actix blocking task. HTTP 404 is already translated into a successful empty segment list, while transport, status, body-read, and JSON parsing failures are returned to reconciliation as retrieval errors. See `proposal.md` for motivation and `specs/sponsorblock-integration/spec.md` for the behavioral contract.

## Goals / Non-Goals

**Goals:**

- Preserve a deterministic endpoint order with the existing hostname first.
- Retry the same request against the alternative hostname after an unusable primary result.
- Keep authoritative empty snapshots distinct from retrieval failures.
- Preserve custom single-endpoint clients used by tests and internal callers.

**Non-Goals:**

- Make endpoint order or hostnames configurable.
- Add concurrent requests, health checks, persistent endpoint state, or retry backoff.
- Change SponsorBlock normalization, caching, or media processing.

## Decisions

### Store an ordered list of base URLs in the client

The default client contains `sponsor.ajay.app` followed by `api.sponsor.ajay.app`. Retrieval iterates over this list and returns the first usable result. The existing constructor continues to create a single-endpoint client, preserving deterministic local tests and avoiding a public configuration change.

Alternative considered: hard-code a second request only in the default fetch path. An endpoint list keeps ordering and retry control in one place and remains straightforward to test.

### Retry after any unusable endpoint result

Transport and timeout errors, non-404 HTTP errors, response-body errors, and malformed JSON allow the next configured endpoint to be tried. These outcomes cannot produce an authoritative snapshot. A warning records the failed endpoint before fallback.

Alternative considered: retry only transport failures. That would leave the service unavailable during upstream 5xx responses or invalid gateway responses even when the alternative endpoint is healthy.

### Treat HTTP 404 as terminal success

SponsorBlock uses 404 to indicate that no matching segments exist. The client returns an empty snapshot immediately and does not query another endpoint, preventing a valid empty state from being mistaken for an outage.

Alternative considered: retry every non-2xx response. This would issue unnecessary requests and weaken the existing authoritative-empty contract.

## Risks / Trade-offs

- [Both endpoints are slow or unavailable, increasing retrieval latency to two attempts] -> Keep the existing per-request timeout and only perform the second attempt after a concrete failure.
- [Fallback can increase request volume during a primary outage] -> Use exactly one ordered fallback without loops or background retries.
- [The two hostnames could temporarily return different snapshots] -> Always prefer a usable primary response and consult the fallback only after primary failure.

## Migration Plan

No data or configuration migration is required. Deploy the client change normally; existing snapshots remain valid. Rollback consists of restoring the single primary endpoint because no persisted format changes are introduced.
