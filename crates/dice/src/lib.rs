/*
 ** EPITECH PROJECT, 2026
 ** lib.rs
 ** File description:
 ** Dice notation parser and roller
 */

pub mod ast;
pub mod error;
pub mod lexer;
pub mod parser;
pub mod roller;

use rand::Rng;

pub use ast::{DiceModifier, Expr, ExprTerm, Operator};
pub use error::DiceError;
pub use roller::RollResult;

pub fn parse(input: &str) -> Result<Expr, DiceError> {
    let tokens = lexer::tokenize(input)?;
    parser::parse(tokens)
}

pub fn roll(input: &str) -> Result<RollResult, DiceError> {
    let expr = parse(input)?;
    let mut rng = rand::rng();
    roller::evaluate(&expr, input, &mut rng)
}

pub fn roll_with_rng(input: &str, rng: &mut impl Rng) -> Result<RollResult, DiceError> {
    let expr = parse(input)?;
    roller::evaluate(&expr, input, rng)
}
