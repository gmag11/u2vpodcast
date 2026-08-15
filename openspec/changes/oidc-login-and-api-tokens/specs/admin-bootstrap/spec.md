## MODIFIED Requirements

### Requirement: Admin bootstrap seeds from config only when both credentials are present

The system SHALL preserve the existing bootstrap behavior only when **both** `admin_username` and `admin_password` are present and non-empty in `config.yml` AND OIDC is not enabled: on every startup it SHALL delete all rows from the `users` table and recreate the single admin account from the config values via `User::default`. If only one of the two fields is present (or empty), the system SHALL ignore both config credentials entirely and run in stored mode: it SHALL NOT modify the `users` table. When `oidc.enabled` is `true`, the system SHALL skip the reseed entirely, leave the `users` table untouched, and rely on just-in-time provisioning from the OIDC provider. This mode SHALL be the default and MUST remain backward compatible with existing configurations that set both fields.

#### Scenario: Both credentials present in config

- **WHEN** `config.yml` sets both `admin_username: admin` and `admin_password: nimda` and OIDC is disabled
- **THEN** on startup all previous `users` rows are removed and a single active admin row with username `admin` and the argon2 hash of `nimda` is created, and logging in with those credentials succeeds

#### Scenario: Only one credential present in config

- **WHEN** `config.yml` sets `admin_username` but omits `admin_password` (or vice versa)
- **THEN** both config credentials are ignored, the `users` table is left untouched, and authentication resolves against the existing database rows as in stored mode

#### Scenario: Reseed skipped when OIDC is enabled

- **WHEN** `oidc.enabled` is `true` and `config.yml` sets both `admin_username` and `admin_password`
- **THEN** the system starts without modifying the `users` table and does not create the admin account from config
