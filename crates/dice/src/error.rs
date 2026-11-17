/*
** EPITECH PROJECT, 2026
** error.rs
** File description:
** Error types for the dice crate
*/

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiceError {
    #[error("Empty expression")]
    EmptyExpression,

    #[error("Invalid token: {0}")]
    InvalidToken(String),

    #[error("Parse error: {0}")]
    ParseError(String),

    #[error("Invalid dice: {0}")]
    InvalidDice(String),

    #[error("Exploding dice limit reached (possible infinite loop on d1)")]
    ExplosionLimit,
}
