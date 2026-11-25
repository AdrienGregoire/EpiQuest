/*
 ** EPITECH PROJECT, 2026
 ** parser.rs
 ** File description:
 ** Parse tokens into an AST
 */

use crate::ast::{DiceModifier, Expr, ExprTerm, Operator};
use crate::error::DiceError;
use crate::lexer::Token;

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Self { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn advance(&mut self) -> Option<&Token> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn expect_number(&mut self) -> Result<u32, DiceError> {
        match self.advance() {
            Some(Token::Number(n)) => Ok(*n),
            other => Err(DiceError::ParseError(format!(
                "Expected a number, got {other:?}"
            ))),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, DiceError> {
        let first_term = self.parse_term()?;
        let mut terms = vec![(Operator::Add, first_term)];

        while let Some(tok) = self.peek() {
            let op = match tok {
                Token::Plus => Operator::Add,
                Token::Minus => Operator::Sub,
                _ => break,
            };
            self.advance();
            let term = self.parse_term()?;
            terms.push((op, term));
        }
        Ok(Expr { terms })
    }

    fn parse_term(&mut self) -> Result<ExprTerm, DiceError> {
        let num = self.expect_number()?;

        if self.peek() == Some(&Token::D) {
            self.advance();
            let sides = self.expect_number()?;

            if sides == 0 {
                return Err(DiceError::InvalidDice(
                    "Number of sides must be at least 1".to_string(),
                ));
            }
            if num == 0 {
                return Err(DiceError::InvalidDice(
                    "Number of dice must be at least 1".to_string(),
                ));
            }

            let modifier = self.parse_modifier(num)?;

            Ok(ExprTerm::Dice {
                count: num,
                sides,
                modifier,
            })
        } else {
            Ok(ExprTerm::Constant(i64::from(num)))
        }
    }

    fn parse_modifier(&mut self, dice_count: u32) -> Result<Option<DiceModifier>, DiceError> {
        match self.peek() {
            Some(Token::Kh) => {
                self.advance();
                let n = self.expect_number()?;
                if n > dice_count {
                    return Err(DiceError::InvalidDice(format!(
                        "Cannot keep {n} dice when only rolling {dice_count}"
                    )));
                }
                Ok(Some(DiceModifier::KeepHighest(n)))
            }
            Some(Token::Kl) => {
                self.advance();
                let n = self.expect_number()?;
                if n > dice_count {
                    return Err(DiceError::InvalidDice(format!(
                        "Cannot keep {n} dice when only rolling {dice_count}"
                    )));
                }
                Ok(Some(DiceModifier::KeepLowest(n)))
            }
            Some(Token::Dh) => {
                self.advance();
                let n = self.expect_number()?;
                if n >= dice_count {
                    return Err(DiceError::InvalidDice(format!(
                        "Cannot drop {n} dice when only rolling {dice_count}"
                    )));
                }
                Ok(Some(DiceModifier::DropHighest(n)))
            }
            Some(Token::Dl) => {
                self.advance();
                let n = self.expect_number()?;
                if n >= dice_count {
                    return Err(DiceError::InvalidDice(format!(
                        "Cannot drop {n} dice when only rolling {dice_count}"
                    )));
                }
                Ok(Some(DiceModifier::DropLowest(n)))
            }
            Some(Token::Bang) => {
                self.advance();
                Ok(Some(DiceModifier::Exploding))
            }
            Some(Token::R) => {
                self.advance();
                let threshold = self.expect_number()?;
                Ok(Some(DiceModifier::Reroll(threshold)))
            }
            _ => Ok(None),
        }
    }
}

pub fn parse(tokens: Vec<Token>) -> Result<Expr, DiceError> {
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;

    if parser.pos < parser.tokens.len() {
        return Err(DiceError::ParseError(format!(
            "Unexpected token at position {}: {:?}",
            parser.pos, parser.tokens[parser.pos]
        )));
    }

    Ok(expr)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::lexer::tokenize;

    #[test]
    fn test_parse_simple() {
        let tokens = tokenize("2d6").unwrap();
        let expr = parse(tokens).unwrap();
        assert_eq!(expr.terms.len(), 1);
        assert_eq!(
            expr.terms[0],
            (
                Operator::Add,
                ExprTerm::Dice {
                    count: 2,
                    sides: 6,
                    modifier: None,
                }
            )
        );
    }

    #[test]
    fn test_parse_with_constant() {
        let tokens = tokenize("2d6+3").unwrap();
        let expr = parse(tokens).unwrap();
        assert_eq!(expr.terms.len(), 2);
        assert_eq!(expr.terms[1], (Operator::Add, ExprTerm::Constant(3)));
    }

    #[test]
    fn test_parse_keep_highest() {
        let tokens = tokenize("4d6kh3").unwrap();
        let expr = parse(tokens).unwrap();
        assert_eq!(
            expr.terms[0],
            (
                Operator::Add,
                ExprTerm::Dice {
                    count: 4,
                    sides: 6,
                    modifier: Some(DiceModifier::KeepHighest(3)),
                }
            )
        );
    }

    #[test]
    fn test_parse_exploding() {
        let tokens = tokenize("1d6!").unwrap();
        let expr = parse(tokens).unwrap();
        assert_eq!(
            expr.terms[0],
            (
                Operator::Add,
                ExprTerm::Dice {
                    count: 1,
                    sides: 6,
                    modifier: Some(DiceModifier::Exploding),
                }
            )
        );
    }

    #[test]
    fn test_parse_subtraction() {
        let tokens = tokenize("1d20-2").unwrap();
        let expr = parse(tokens).unwrap();
        assert_eq!(expr.terms.len(), 2);
        assert_eq!(expr.terms[1], (Operator::Sub, ExprTerm::Constant(2)));
    }

    #[test]
    fn test_parse_complex() {
        let tokens = tokenize("2d6+1d8+5").unwrap();
        let expr = parse(tokens).unwrap();
        assert_eq!(expr.terms.len(), 3);
    }

    #[test]
    fn test_parse_zero_sides() {
        let tokens = tokenize("1d0").unwrap();
        assert!(parse(tokens).is_err());
    }

    #[test]
    fn test_parse_zero_count() {
        let tokens = tokenize("0d6").unwrap();
        assert!(parse(tokens).is_err());
    }

    #[test]
    fn test_parse_keep_too_many() {
        let tokens = tokenize("2d6kh5").unwrap();
        assert!(parse(tokens).is_err());
    }

    #[test]
    fn test_parse_drop_all() {
        let tokens = tokenize("2d6dl2").unwrap();
        assert!(parse(tokens).is_err());
    }
}
