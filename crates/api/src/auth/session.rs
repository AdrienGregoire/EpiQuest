/*
** EPITECH PROJECT, 2026
** session.rs
** File description:
** session token helpers
*/

use rand_core::{OsRng, RngCore};
use sha2::{Digest, Sha256};
use std::fmt::Write;

pub const SESSION_COOKIE: &str = "session";
pub const SESSION_TTL_SECS: i64 = 7 * 24 * 60 * 60;

fn to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(bytes.len() * 2), |mut out, b| {
            let _ = write!(out, "{b:02x}");
            out
        })
}

pub fn generate_token() -> String {
    let mut bytes = [0u8; 32];

    OsRng.fill_bytes(&mut bytes);
    to_hex(&bytes)
}

pub fn hash_token(token: &str) -> String {
    to_hex(&Sha256::digest(token.as_bytes()))
}

pub fn cookie_secure() -> bool {
    std::env::var("COOKIE_SECURE")
        .map(|v| v != "false")
        .unwrap_or(true)
}

//Unit Tests
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_has_expected_length() {
        assert_eq!(generate_token().len(), 64);
    }

    #[test]
    fn test_tokens_are_unique() {
        assert_ne!(generate_token(), generate_token());
    }

    #[test]
    fn test_hash_is_deterministic_and_differs_from_token() {
        let token = generate_token();

        assert_eq!(hash_token(&token), hash_token(&token));
        assert_ne!(hash_token(&token), token);
    }
}
