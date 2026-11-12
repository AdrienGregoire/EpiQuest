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
