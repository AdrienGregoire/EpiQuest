/*
 ** EPITECH PROJECT, 2026
 ** parser.rs
 ** File description:
 ** Parse tokens into an AST
 */

use crate::ast::{DiceModifier, Expr, ExprTerm, Operator};
use crate::error::{DiceError, Span, SyntaxError};
use crate::lexer::{SpannedToken, Token};

struct Parser {
    tokens: Vec<SpannedToken>,
    pos: usize,
    eof: Span,
}

impl Parser {
    fn new(tokens: Vec<SpannedToken>) -> Self {
        let eof = Span::at(tokens.last().map_or(0, |t| t.span.end));
        Self {
            tokens,
            pos: 0,
            eof,
        }
    }

    fn peek(&self) -> Option<&SpannedToken> {
        self.tokens.get(self.pos)
    }

    fn peek_token(&self) -> Option<&Token> {
        self.peek().map(|t| &t.token)
    }

    fn advance(&mut self) -> Option<&SpannedToken> {
        let tok = self.tokens.get(self.pos);
        if tok.is_some() {
            self.pos += 1;
        }
        tok
    }

    fn error_here(&self, message: impl Into<String>, hint: impl Into<String>) -> DiceError {
        let span = self.peek().map_or(self.eof, |t| t.span);
        DiceError::Syntax(SyntaxError::new(message, span).with_hint(hint))
    }

    fn expected(&self, what: &str, hint: &str) -> DiceError {
        let found = match self.peek_token() {
            Some(tok) => format!("found '{tok}'"),
            None => "but the expression ends here".to_string(),
        };
        self.error_here(format!("Expected {what}, {found}"), hint)
    }

    fn expect_number(&mut self, what: &str, hint: &str) -> Result<(u32, Span), DiceError> {
        match self.peek() {
            Some(SpannedToken {
                token: Token::Number(n),
                span,
            }) => {
                let found = (*n, *span);
                self.pos += 1;
                Ok(found)
            }
            _ => Err(self.expected(what, hint)),
        }
    }

    fn reject_modifier(
        &self,
        message: impl FnOnce(&Token) -> String,
        hint: &str,
    ) -> Result<(), DiceError> {
        match self.peek_token() {
            Some(tok) if tok.is_modifier() => Err(self.error_here(message(tok), hint)),
            _ => Ok(()),
        }
    }

    fn parse_expr(&mut self) -> Result<Expr, DiceError> {
        let first_term = self.parse_term()?;
        let mut terms = vec![(Operator::Add, first_term)];

        while let Some(tok) = self.peek_token() {
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
        if self.peek_token() == Some(&Token::D) {
            return Err(self.error_here(
                "Missing number of dice before 'd'",
                "Write 1d20 instead of d20",
            ));
        }

        let (num, num_span) = self.expect_number(
            "a number or a dice roll",
            "Write a number (3) or dice (1d6), e.g. 2d6+3",
        )?;

        if self.peek_token() != Some(&Token::D) {
            self.reject_modifier(
                |tok| format!("'{tok}' can only follow dice, not a plain number"),
                "Example: 4d6kh3",
            )?;
            return Ok(ExprTerm::Constant(i64::from(num)));
        }
        self.advance();

        let (sides, sides_span) = self.expect_number(
            "the number of sides after 'd'",
            "Example: 2d6 rolls two six-sided dice",
        )?;

        if sides == 0 {
            return Err(DiceError::Syntax(
                SyntaxError::new("A die needs at least 1 side", sides_span)
                    .with_hint("Example: 1d6"),
            ));
        }
        if num == 0 {
            return Err(DiceError::Syntax(
                SyntaxError::new("You must roll at least 1 die", num_span)
                    .with_hint("Example: 1d6"),
            ));
        }

        let modifier = self.parse_modifier(num, sides)?;
        self.reject_modifier(
            |tok| {
                format!(
                    "Only one modifier (kh, kl, dh, dl, ! or r) is allowed per dice group, '{tok}' is extra"
                )
            },
            "Keep a single modifier, e.g. 4d6kh3",
        )?;

        Ok(ExprTerm::Dice {
            count: num,
            sides,
            modifier,
        })
    }

    fn parse_modifier(
        &mut self,
        dice_count: u32,
        sides: u32,
    ) -> Result<Option<DiceModifier>, DiceError> {
        let Some(SpannedToken { token, span }) = self.peek().cloned() else {
            return Ok(None);
        };
        let modifier = match token {
            Token::Kh => DiceModifier::KeepHighest(self.keep_count(&token, dice_count)?),
            Token::Kl => DiceModifier::KeepLowest(self.keep_count(&token, dice_count)?),
            Token::Dh => DiceModifier::DropHighest(self.drop_count(&token, dice_count)?),
            Token::Dl => DiceModifier::DropLowest(self.drop_count(&token, dice_count)?),
            Token::Bang => {
                if sides == 1 {
                    return Err(DiceError::Syntax(
                        SyntaxError::new("Exploding dice need at least 2 sides", span).with_hint(
                            "A 1-sided die always rolls its maximum, so it would explode forever",
                        ),
                    ));
                }
                self.advance();
                DiceModifier::Exploding
            }
            Token::R => DiceModifier::Reroll(self.number_after(&token)?.0),
            _ => return Ok(None),
        };
        Ok(Some(modifier))
    }

    fn number_after(&mut self, token: &Token) -> Result<(u32, Span), DiceError> {
        self.advance();
        let (what, hint) = match token {
            Token::Kh => (
                "the number of dice to keep after 'kh'",
                "Example: 4d6kh3 keeps the 3 highest dice",
            ),
            Token::Kl => (
                "the number of dice to keep after 'kl'",
                "Example: 2d20kl1 keeps the lowest die",
            ),
            Token::Dh => (
                "the number of dice to drop after 'dh'",
                "Example: 4d6dh1 drops the highest die",
            ),
            Token::Dl => (
                "the number of dice to drop after 'dl'",
                "Example: 4d6dl1 drops the lowest die",
            ),
            _ => (
                "the value to reroll after 'r'",
                "Example: 1d6r1 rerolls a 1 (once)",
            ),
        };
        self.expect_number(what, hint)
    }

    fn keep_count(&mut self, token: &Token, dice_count: u32) -> Result<u32, DiceError> {
        let (n, n_span) = self.number_after(token)?;
        if n > dice_count {
            return Err(DiceError::Syntax(
                SyntaxError::new(
                    format!("Cannot keep {n} dice when only rolling {dice_count}"),
                    n_span,
                )
                .with_hint(format!("Keep at most {dice_count}")),
            ));
        }
        Ok(n)
    }

    fn drop_count(&mut self, token: &Token, dice_count: u32) -> Result<u32, DiceError> {
        let (n, n_span) = self.number_after(token)?;
        if n >= dice_count {
            return Err(DiceError::Syntax(
                SyntaxError::new(
                    format!("Cannot drop {n} dice when only rolling {dice_count}"),
                    n_span,
                )
                .with_hint(format!(
                    "At least one die must remain: drop at most {}",
                    dice_count - 1
                )),
            ));
        }
        Ok(n)
    }
}

pub fn parse(tokens: Vec<SpannedToken>) -> Result<Expr, DiceError> {
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;

    match parser.peek_token() {
        None => Ok(expr),
        Some(tok) => Err(parser.error_here(
            format!("Missing '+' or '-' before '{tok}'"),
            "Join terms with + or -, e.g. 2d6+3",
        )),
    }
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
