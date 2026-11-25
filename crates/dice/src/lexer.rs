/*
 ** EPITECH PROJECT, 2026
 ** lexer.rs
 ** File description:
 ** Tokenizer for dice notation strings
 */

use std::fmt;

use crate::error::{DiceError, Span, SyntaxError};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Token {
    Number(u32),
    D,
    Plus,
    Minus,
    Kh,   //Keep Highest
    Kl,   //Keep Lowest
    Dh,   //Drop Highest
    Dl,   //Drop Lowest
    Bang, //For exploding dice
    R,    //Reroll
}

impl Token {
    #[must_use]
    pub fn is_modifier(&self) -> bool {
        matches!(
            self,
            Token::Kh | Token::Kl | Token::Dh | Token::Dl | Token::Bang | Token::R
        )
    }
}

impl fmt::Display for Token {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Token::Number(n) => write!(f, "{n}"),
            Token::D => write!(f, "d"),
            Token::Plus => write!(f, "+"),
            Token::Minus => write!(f, "-"),
            Token::Kh => write!(f, "kh"),
            Token::Kl => write!(f, "kl"),
            Token::Dh => write!(f, "dh"),
            Token::Dl => write!(f, "dl"),
            Token::Bang => write!(f, "!"),
            Token::R => write!(f, "r"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SpannedToken {
    pub token: Token,
    pub span: Span,
}

pub fn tokenize(input: &str) -> Result<Vec<SpannedToken>, DiceError> {
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];
        if c.is_whitespace() {
            i += 1;
            continue;
        }
        let next = chars.get(i + 1).map(char::to_ascii_lowercase);
        let (token, len) = match c.to_ascii_lowercase() {
            '+' => (Token::Plus, 1),
            '-' => (Token::Minus, 1),
            '!' => (Token::Bang, 1),
            'r' => (Token::R, 1),
            'k' => match next {
                Some('h') => (Token::Kh, 2),
                Some('l') => (Token::Kl, 2),
                _ => return Err(lone_k_error(i)),
            },
            'd' => match next {
                Some('h') => (Token::Dh, 2),
                Some('l') => (Token::Dl, 2),
                _ => (Token::D, 1),
            },
            '0'..='9' => {
                let len = chars[i..].iter().take_while(|c| c.is_ascii_digit()).count();
                let digits: String = chars[i..i + len].iter().collect();
                let num = digits
                    .parse::<u32>()
                    .map_err(|_| number_too_large_error(&digits, i))?;
                (Token::Number(num), len)
            }
            _ => return Err(unexpected_char_error(c, i)),
        };
        tokens.push(SpannedToken {
            token,
            span: Span::new(i, i + len),
        });
        i += len;
    }
    if tokens.is_empty() {
        return Err(DiceError::EmptyExpression);
    }
    Ok(tokens)
}

fn lone_k_error(pos: usize) -> DiceError {
    DiceError::Syntax(
        SyntaxError::new(
            "'k' must be followed by 'h' or 'l'",
            Span::new(pos, pos + 1),
        )
        .with_hint("Use kh to keep the highest dice or kl to keep the lowest, e.g. 4d6kh3"),
    )
}

fn number_too_large_error(digits: &str, pos: usize) -> DiceError {
    DiceError::Syntax(
        SyntaxError::new(
            format!("Number '{digits}' is too large"),
            Span::new(pos, pos + digits.len()),
        )
        .with_hint(format!("Use a number up to {}", u32::MAX)),
    )
}

fn unexpected_char_error(c: char, pos: usize) -> DiceError {
    let hint = match c {
        '*' | '/' | '(' | ')' => "Only addition and subtraction are supported, e.g. 2d6+3",
        _ => "Allowed: digits, d, +, -, kh, kl, dh, dl, ! and r, e.g. 4d6kh3+2",
    };
    DiceError::Syntax(
        SyntaxError::new(
            format!("Unexpected character '{c}'"),
            Span::new(pos, pos + 1),
        )
        .with_hint(hint),
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    fn kinds(input: &str) -> Vec<Token> {
        tokenize(input)
            .unwrap()
            .into_iter()
            .map(|t| t.token)
            .collect()
    }

    #[test]
    fn test_simple_roll() {
        let tokens = kinds("2d6");
        assert_eq!(tokens, vec![Token::Number(2), Token::D, Token::Number(6)]);
    }

    #[test]
    fn test_roll_with_modifier() {
        let tokens = kinds("2d6+3");
        assert_eq!(
            tokens,
            vec![
                Token::Number(2),
                Token::D,
                Token::Number(6),
                Token::Plus,
                Token::Number(3)
            ]
        );
    }

    #[test]
    fn test_keep_highest() {
        let tokens = kinds("4d6kh3");
        assert_eq!(
            tokens,
            vec![
                Token::Number(4),
                Token::D,
                Token::Number(6),
                Token::Kh,
                Token::Number(3)
            ]
        );
    }

    #[test]
    fn test_exploding() {
        let tokens = kinds("1d6!");
        assert_eq!(
            tokens,
            vec![Token::Number(1), Token::D, Token::Number(6), Token::Bang]
        );
    }

    #[test]
    fn test_reroll() {
        let tokens = kinds("1d6r1");
        assert_eq!(
            tokens,
            vec![
                Token::Number(1),
                Token::D,
                Token::Number(6),
                Token::R,
                Token::Number(1)
            ]
        );
    }

    #[test]
    fn test_drop_lowest() {
        let tokens = kinds("4d6dl1");
        assert_eq!(
            tokens,
            vec![
                Token::Number(4),
                Token::D,
                Token::Number(6),
                Token::Dl,
                Token::Number(1)
            ]
        );
    }

    #[test]
    fn test_case_insensitive() {
        let tokens = kinds("2D6KH1");
        assert_eq!(
            tokens,
            vec![
                Token::Number(2),
                Token::D,
                Token::Number(6),
                Token::Kh,
                Token::Number(1)
            ]
        );
    }

    #[test]
    fn test_complex_expression() {
        let tokens = kinds("2d6+1d8-2");
        assert_eq!(
            tokens,
            vec![
                Token::Number(2),
                Token::D,
                Token::Number(6),
                Token::Plus,
                Token::Number(1),
                Token::D,
                Token::Number(8),
                Token::Minus,
                Token::Number(2)
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        assert!(tokenize("").is_err());
        assert!(tokenize("   ").is_err());
    }

    #[test]
    fn test_invalid_character() {
        assert!(tokenize("2d6@3").is_err());
    }

    #[test]
    fn test_spans() {
        let tokens = tokenize("2d6+3").unwrap();
        let spans: Vec<(usize, usize)> =
            tokens.iter().map(|t| (t.span.start, t.span.end)).collect();
        assert_eq!(spans, vec![(0, 1), (1, 2), (2, 3), (3, 4), (4, 5)]);
    }

    #[test]
    fn test_spans_two_chars_and_multi_digits() {
        let tokens = tokenize("10d20kh3").unwrap();
        let spans: Vec<(usize, usize)> =
            tokens.iter().map(|t| (t.span.start, t.span.end)).collect();
        assert_eq!(spans, vec![(0, 2), (2, 3), (3, 5), (5, 7), (7, 8)]);
    }

    #[test]
    fn test_spans_ignore_surrounding_whitespace() {
        let tokens = tokenize("  2d6 + 3").unwrap();
        assert_eq!(tokens[0].span, Span::new(2, 3));
        assert_eq!(tokens[3].span, Span::new(6, 7));
        assert_eq!(tokens[4].span, Span::new(8, 9));
    }

    #[test]
    fn test_tab_is_whitespace() {
        assert_eq!(kinds("2d6\t+3").len(), 5);
    }

    fn syntax_error(input: &str) -> SyntaxError {
        match tokenize(input).unwrap_err() {
            DiceError::Syntax(err) => err,
            other => panic!("expected a syntax error, got {other:?}"),
        }
    }

    #[test]
    fn test_unexpected_character_error() {
        let err = syntax_error("2d6@3");
        assert_eq!(err.message, "Unexpected character '@'");
        assert_eq!(err.span, Span::new(3, 4));
        assert!(err.hint.is_some());
    }

    #[test]
    fn test_unexpected_character_keeps_original_case_and_position() {
        let err = syntax_error("  2D6 É");
        assert_eq!(err.message, "Unexpected character 'É'");
        assert_eq!(err.span, Span::new(6, 7));
    }

    #[test]
    fn test_unsupported_operator_hint() {
        let err = syntax_error("2d6*2");
        assert!(err.hint.unwrap().contains("addition and subtraction"));
    }

    #[test]
    fn test_lone_k_error() {
        let err = syntax_error("4d6k3");
        assert_eq!(err.message, "'k' must be followed by 'h' or 'l'");
        assert_eq!(err.span, Span::new(3, 4));
    }

    #[test]
    fn test_number_too_large_error() {
        let err = syntax_error("99999999999d6");
        assert_eq!(err.message, "Number '99999999999' is too large");
        assert_eq!(err.span, Span::new(0, 11));
    }
}
