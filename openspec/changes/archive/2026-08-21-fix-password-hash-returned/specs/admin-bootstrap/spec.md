## ADDED Requirements

### Requirement: API responses never serialize the stored password hash

No API response (list users, single user, create, delete, session payload, or any future `User`-carrying payload) SHALL include the `hashed_password` value. The password hash SHALL remain available only inside the server for `verify_password` checks. The serialized shape of every existing `User`-based response SHALL remain unchanged except for the removal of the hash field, so the SPA keeps working.

#### Scenario: User list omits hashes
- **WHEN** an authenticated client sends `GET /api/1.0/users/`
- **THEN** the response contains each user's id, name, role and active flag but no `hashed_password` key or value

#### Scenario: User create response omits the hash
- **WHEN** a user is created and the creation response serializes the new row
- **THEN** the response contains no `hashed_password` field

#### Scenario: Password verification still works server-side
- **WHEN** a client logs in with a valid password after the serialization change
- **THEN** authentication succeeds because the hash is still read from the database row internally, even though no response ever carried it