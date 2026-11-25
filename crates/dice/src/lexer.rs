/*
 ** EPITECH PROJECT, 2026
 ** lexer.rs
 ** File description:
 ** Tokenizer for dice notation strings
 */

use crate::error::DiceError;

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

pub fn tokenize(input: &str) -> Result<Vec<Token>, DiceError> {
    let input = input.trim().to_lowercase();
    let chars: Vec<char> = input.chars().collect();
    let mut tokens = Vec::new();
    let mut i = 0;

    while i < chars.len() {
        match chars[i] {
            ' ' => {
                i += 1;
            }
            '+' => {
                tokens.push(Token::Plus);
                i += 1;
            }
            '-' => {
                tokens.push(Token::Minus);
                i += 1;
            }
            '!' => {
                tokens.push(Token::Bang);
                i += 1;
            }
            'k' => {
                if i + 1 < chars.len() && chars[i + 1] == 'h' {
                    tokens.push(Token::Kh);
                    i += 2;
                } else if i + 1 < chars.len() && chars[i + 1] == 'l' {
                    tokens.push(Token::Kl);
                    i += 2;
                } else {
                    return Err(DiceError::InvalidToken(format!(
                        "Expected 'kh' or 'kl' at position {i}"
                    )));
                }
            }
            'd' => {
                if i + 1 < chars.len() && chars[i + 1] == 'h' {
                    tokens.push(Token::Dh);
                    i += 2;
                } else if i + 1 < chars.len() && chars[i + 1] == 'l' {
                    tokens.push(Token::Dl);
                    i += 2;
                } else {
                    tokens.push(Token::D);
                    i += 1;
                }
            }
            'r' => {
                tokens.push(Token::R);
                i += 1;
            }
            c if c.is_ascii_digit() => {
                let start = i;
                while i < chars.len() && chars[i].is_ascii_digit() {
                    i += 1;
                }
                let num_str: String = chars[start..i].iter().collect();
                let num = num_str
                    .parse::<u32>()
                    .map_err(|_| DiceError::InvalidToken(format!("Number too large: {num_str}")))?;
                tokens.push(Token::Number(num));
            }
            c => {
                return Err(DiceError::InvalidToken(format!(
                    "Unexpected character '{c}' at position {i}"
                )));
            }
        }
    }
    if tokens.is_empty() {
        return Err(DiceError::EmptyExpression);
    }
    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_roll() {
        let tokens = tokenize("2d6").unwrap();
        assert_eq!(tokens, vec![Token::Number(2), Token::D, Token::Number(6)]);
    }

    #[test]
    fn test_roll_with_modifier() {
        let tokens = tokenize("2d6+3").unwrap();
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
        let tokens = tokenize("4d6kh3").unwrap();
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
        let tokens = tokenize("1d6!").unwrap();
        assert_eq!(
            tokens,
            vec![Token::Number(1), Token::D, Token::Number(6), Token::Bang]
        );
    }

    #[test]
    fn test_reroll() {
        let tokens = tokenize("1d6r1").unwrap();
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
        let tokens = tokenize("4d6dl1").unwrap();
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
        let tokens = tokenize("2D6KH1").unwrap();
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
        let tokens = tokenize("2d6+1d8-2").unwrap();
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
}
