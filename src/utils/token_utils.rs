use argon2::{
    password_hash::{phc::SaltString, PasswordHasher, PasswordVerifier},
    Argon2,
};

pub async fn hash_password(password: &str) -> String {
    let salt = SaltString::generate();
    Argon2::default()
        .hash_password_with_salt(password.as_bytes(), salt.as_bytes())
        .expect("Unable to hash password")
        .to_string()
}

pub async fn verify_password(
    password: &str,
    hash: &str,
) -> Result<(), argon2::password_hash::Error> {
    Argon2::default().verify_password(password.as_bytes(), hash)
}
