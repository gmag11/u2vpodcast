## 1. Restrict development CORS

- [x] 1.1 Extract a shared CORS builder (methods/headers/expose/max_age + `supports_credentials`) usable by both production and development branches
- [x] 1.2 Build the development allowlist: configured `url`, `http://localhost:{port}`, `http://127.0.0.1:{port}`, `https://yt3.googleusercontent.com`
- [x] 1.3 Validate every non-constant dev origin via `validate_origin` at startup with fail-fast errors
- [x] 1.4 Remove `allow_any_origin()` from the development branch; confirm no mode combines a wildcard with credentials

## 2. Tests

- [x] 2.1 Extend `cors_tests` in `src/main.rs` with the localhost/127.0.0.1 origin strings (accepted) and a port-less/invalid variant (rejected)
- [x] 2.2 Dev-mode functional check: SPA login + channel requests still work from `http://localhost:{port}` (credentials flow intact)
- [x] 2.3 Negative check: `Origin: http://evil.example` in dev mode is not echoed and credentials are not granted

## 3. Verification

- [x] 3.1 Production CORS scenarios from `api-cors-policy` still hold (no regressions)
- [x] 3.2 Full test suite passes