/*
** EPITECH PROJECT, 2026
** password.rs
** File description:
** password functions
*/

use argon2::{
    Argon2,
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
};
use rand_core::OsRng;

pub fn hash_password(plain: &str) -> Result<String, argon2::password_hash::Error> {
    let salt = SaltString::generate(&mut OsRng);
    let argon2 = Argon2::default();
    let hashed_password = argon2.hash_password(plain.as_bytes(), &salt)?.to_string();

    Ok(hashed_password)
}

pub fn verify_password(plain: &str, hash: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;

    Ok(Argon2::default()
        .verify_password(plain.as_bytes(), &parsed_hash)
        .is_ok())
}

// Unit Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hash_not_equal_to_plain() {
        let password = "my_super_secret_password";
        let hash = hash_password(password).expect("hashing failed");

        assert_ne!(hash, password);
    }

    #[test]
    fn test_verify_password_ok() {
        let password = "my_super_secret_password";
        let hash = hash_password(password).expect("hashing failed");

        assert!(verify_password(password, &hash).expect("verify failed"));
    }

    #[test]
    fn test_verify_password_wrong() {
        let password = "my_super_secret_password";
        let hash = hash_password(password).expect("hashing failed");

        assert!(!verify_password("wrong_password", &hash).expect("verify failed"));
    }

    #[test]
    fn test_two_hashes_differ_due_to_salt() {
        let password = "same_password";
        let hash1 = hash_password(password).expect("hash 1 failed");
        let hash2 = hash_password(password).expect("hash 2 failed");

        assert_ne!(
            hash1, hash2,
            "Two hashes of the same password must differ (salt)"
        );
    }
}
