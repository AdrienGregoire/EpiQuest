/*
 ** EPITECH PROJECT, 2026
 ** error.rs
 ** File description:
 ** Error types for the dice crate
 */

use std::fmt;

use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Span {
    pub start: usize,
    pub end: usize,
}

impl Span {
    #[must_use]
    pub fn new(start: usize, end: usize) -> Self {
        Self { start, end }
    }

    #[must_use]
    pub fn at(pos: usize) -> Self {
        Self::new(pos, pos)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SyntaxError {
    pub message: String,
    pub span: Span,
    pub hint: Option<String>,
}

impl SyntaxError {
    pub fn new(message: impl Into<String>, span: Span) -> Self {
        Self {
            message: message.into(),
            span,
            hint: None,
        }
    }

    #[must_use]
    pub fn with_hint(mut self, hint: impl Into<String>) -> Self {
        self.hint = Some(hint.into());
        self
    }

    #[must_use]
    pub fn explain(&self, input: &str) -> String {
        let width = self.span.end.saturating_sub(self.span.start).max(1);
        let mut out = format!(
            "{}\n{}\n{}{}",
            self.message,
            input,
            " ".repeat(self.span.start),
            "^".repeat(width)
        );
        if let Some(hint) = &self.hint {
            out.push_str("\nHint: ");
            out.push_str(hint);
        }
        out
    }
}

impl fmt::Display for SyntaxError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} (at character {})", self.message, self.span.start + 1)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Error)]
pub enum DiceError {
    #[error("Empty expression: type a dice roll, e.g. 2d6+3")]
    EmptyExpression,

    #[error("{0}")]
    Syntax(SyntaxError),

    #[error(
        "Exploding dice limit reached (a 1-sided die always rolls its maximum and would explode forever)"
    )]
    ExplosionLimit,
}

impl DiceError {
    #[must_use]
    pub fn explain(&self, input: &str) -> String {
        match self {
            DiceError::Syntax(err) => err.explain(input),
            other => other.to_string(),
        }
    }
}
