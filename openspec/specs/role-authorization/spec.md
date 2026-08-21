# role-authorization

## Purpose

Documents the authorization model for the current single-admin deployment and the contract for the future role-management stage. Today the system operates with exactly one administrator account, so every authenticated session has full powers; role-based enforcement is intentionally deferred until user/role management exists.

## Requirements

### Requirement: Current deployment is single-user, all-admin

The system SHALL operate with a single user account that acts as administrator. User-management API endpoints (`/api/1.0/users/**`) exist, but the deployment holds exactly one account and there is no role-management flow, so every successful authenticated session SHALL have full application powers (channel management, options, and the user endpoints themselves) because every existing account is already an administrator. Role-based authorization is intentionally not enforced in this iteration.

#### Scenario: Only one account exists and it can do everything
- **WHEN** the app runs with the seeded admin account and the operator logs in
- **THEN** the session is authorized for every route including user-management endpoints, and no other account exists

#### Scenario: User-management endpoints serve only the single existing account
- **WHEN** a client calls the user-management endpoints while the deployment holds a single admin account
- **THEN** they address or return that single account; there is no multi-account role flow to guard yet

### Requirement: Role enforcement is deferred to the role-management stage

Role-based authorization SHALL NOT be implemented piecemeal while no role-management system exists. The future stage that introduces user creation/deletion and roles SHALL also enforce authorization on user-management endpoints and on admin-only operations (e.g. full re-sync of all channels) via the session role claim, reject requests from non-admin roles, and prevent deleting the last active administrator. This change SHALL be revisited and built together with that stage.

#### Scenario: Future stage enables user management with guards
- **WHEN** a future release adds user create/delete/role endpoints
- **THEN** user-management endpoints SHALL require an admin role, non-admin sessions SHALL be rejected, and deleting the last active admin SHALL be refused

#### Scenario: Current iteration keeps single-admin behavior
- **WHEN** the app runs before the role-management stage exists
- **THEN** authorization continues to rely on the single all-admin model, with no partial role checks introduced in the interim
