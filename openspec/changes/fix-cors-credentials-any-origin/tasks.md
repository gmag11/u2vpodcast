## 1. CORS Hardening

- [ ] 1.1 Remove `.allow_any_origin()` from the production CORS branch in `src/main.rs`
- [ ] 1.2 Correct `allowed_origin(...)` entries: no trailing slash, full `scheme://host[:port]` origins (configured `url` plus the YouTube image host)
- [ ] 1.3 Add startup validation of every production allowlist origin; abort with a clear error on invalid input; update sample configs to full origins

## 2. Verification & Regression

- [ ] 2.1 Build and start the app; run curl preflight/simple requests with an allowed and a disallowed `Origin` and assert the CORS response headers
- [ ] 2.2 Run the full test suite, then re-run a bug review taking `docs/bug-review-2026-08-21.md` as reference; confirm bug #1 is resolved and no new bugs were introduced