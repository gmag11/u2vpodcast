use serde::{Serialize, Deserialize};
use std::{
    fmt,
    slice::Iter
};

/// Application role of a user account.
///
/// IMPORTANT (single-admin invariant): there is no role-management system yet.
/// The deployment holds exactly one account, which is always an administrator,
/// and no handler enforces per-route role checks on purpose. Role-based
/// authorization SHALL be introduced together with the future role-management
/// stage, which must enforce admin checks on the user-management endpoints,
/// reject non-admin sessions, and prevent deleting the last active admin. See
/// openspec change `defer-role-authorization` (`role-authorization` capability).
#[derive(sqlx::Type)]
#[sqlx(rename_all = "lowercase")]
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq)]
pub enum Role {
    User,
    Admin,
}

impl Role {
    #[allow(unused)]
    fn iterator() -> Iter<'static, Role>{
        static ROLES: [Role; 2] = [Role::User, Role::Admin];
        ROLES.iter()
    }

    #[allow(unused)]
    pub fn get_roles() -> Vec<String>{
        let mut roles = Vec::new();
        for role in Role::iterator(){
            roles.push(format!("{role}"));
        }
        roles
    }
}

impl fmt::Display for Role{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Role::User => write!(f, "User"),
            Role::Admin => write!(f, "Admin"),
        }
    }
}
