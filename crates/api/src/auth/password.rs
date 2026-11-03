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
