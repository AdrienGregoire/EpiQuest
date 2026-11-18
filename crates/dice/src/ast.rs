/*
** EPITECH PROJECT, 2026
** ast.rs
** File description:
** Abstract Syntax Tree types for dice notation
*/

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DiceModifier {
    KeepHighest(u32),
    KeepLowest(u32),
    DropHighest(u32),
    DropLowest(u32),
    Exploding,
    Reroll(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Operator {
    Add,
    Sub,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExprTerm {
    Dice {
        count: u32,
        sides: u32,
        modifier: Option<DiceModifier>,
    },
    Constant(i64),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Expr {
    pub terms: Vec<(Operator, ExprTerm)>,
}
