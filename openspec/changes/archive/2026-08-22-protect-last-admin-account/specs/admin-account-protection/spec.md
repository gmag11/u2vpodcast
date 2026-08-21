## Purpose

Defines the safety invariant that the last active administrator account can never be deleted through the API, protecting the deployment from unrecoverable lockout. This is a single sub-guarantee of the deferred `role-authorization` stage, implemented now without introducing role management.

## ADDED Requirements

### Requirement: The last active administrator cannot be deleted

The user-deletion API SHALL refuse to delete an active admin when no other active admin remains afterwards. The refusal SHALL return a non-2xx status (e.g. `409 Conflict`) with a clear message, and SHALL leave the user row intact. The guard SHALL apply regardless of which session performs the deletion.

#### Scenario: Deleting the only admin is refused
- **WHEN** the deployment holds one active administrator and a delete request targets that admin
- **THEN** the request fails with `409` (or an explicit 4xx) and the admin row is unchanged

#### Scenario: Deleting an admin when another remains succeeds
- **WHEN** two active admins exist and a delete request targets one of them
- **THEN** the deletion succeeds and the other admin remains

#### Scenario: Non-admin deletion is unaffected
- **WHEN** a regular (non-admin) user is deleted and at least one admin exists
- **THEN** the deletion succeeds as before

#### Scenario: Startup seeding remains unaffected
- **WHEN** the app boots with `admin_username`/`admin_password` configured and reseeds the users table
- **THEN** seeding proceeds regardless of this guard