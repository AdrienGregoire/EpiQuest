/*
 ** EPITECH PROJECT, 2026
 ** errors.rs
 ** File description:
 ** Integration tests for explained parse errors
 */

use dice::{DiceError, Span, SyntaxError, parse, roll};

fn syntax_error(input: &str) -> SyntaxError {
    match parse(input).unwrap_err() {
        DiceError::Syntax(err) => err,
        other => panic!("expected a syntax error for {input:?}, got {other:?}"),
    }
}

const CASES: &[(&str, &str, usize, usize)] = &[
    // Lexer
    ("2d6@3", "Unexpected character '@'", 3, 4),
    ("2d6*2", "Unexpected character '*'", 3, 4),
    ("  2d6 @", "Unexpected character '@'", 6, 7),
    ("4d6k3", "'k' must be followed by 'h' or 'l'", 3, 4),
    ("99999999999d6", "Number '99999999999' is too large", 0, 11),
    // Missing pieces
    (
        "2d6+",
        "Expected a number or a dice roll, but the expression ends here",
        4,
        4,
    ),
    (
        "2d6++3",
        "Expected a number or a dice roll, found '+'",
        4,
        5,
    ),
    ("d20", "Missing number of dice before 'd'", 0, 1),
    (
        "2d",
        "Expected the number of sides after 'd', but the expression ends here",
        2,
        2,
    ),
    (
        "2d+3",
        "Expected the number of sides after 'd', found '+'",
        2,
        3,
    ),
    (
        "4d6kh",
        "Expected the number of dice to keep after 'kh', but the expression ends here",
        5,
        5,
    ),
    (
        "1d6r+1",
        "Expected the value to reroll after 'r', found '+'",
        4,
        5,
    ),
    // Values that make no sense
    ("1d0", "A die needs at least 1 side", 2, 3),
    ("0d6", "You must roll at least 1 die", 0, 1),
    ("2d6kh5", "Cannot keep 5 dice when only rolling 2", 5, 6),
    ("2d6dl2", "Cannot drop 2 dice when only rolling 2", 5, 6),
    ("1d1!", "Exploding dice need at least 2 sides", 3, 4),
    // Misplaced pieces
    (
        "4d6kh3!",
        "Only one modifier (kh, kl, dh, dl, ! or r) is allowed per dice group, '!' is extra",
        6,
        7,
    ),
    (
        "5kh2",
        "'kh' can only follow dice, not a plain number",
        1,
        3,
    ),
    ("2d6 3", "Missing '+' or '-' before '3'", 4, 5),
];

#[test]
fn test_error_messages_and_positions() {
    for &(input, message, start, end) in CASES {
        let err = syntax_error(input);
        assert_eq!(err.message, message, "message for {input:?}");
        assert_eq!(err.span, Span::new(start, end), "span for {input:?}");
    }
}

#[test]
fn test_every_error_has_a_hint() {
    for &(input, _, _, _) in CASES {
        let err = syntax_error(input);
        assert!(
            err.hint.is_some_and(|h| !h.is_empty()),
            "hint for {input:?}"
        );
    }
}

#[test]
fn test_empty_input() {
    assert_eq!(parse("").unwrap_err(), DiceError::EmptyExpression);
    assert_eq!(parse("   ").unwrap_err(), DiceError::EmptyExpression);
}

#[test]
fn test_display_gives_a_one_based_position() {
    let err = parse("2d6@3").unwrap_err();
    assert_eq!(err.to_string(), "Unexpected character '@' (at character 4)");
}

#[test]
fn test_explain_points_at_the_problem() {
    let err = parse("2d6@3").unwrap_err();
    assert_eq!(
        err.explain("2d6@3"),
        "Unexpected character '@'\n\
         2d6@3\n\
         \x20  ^\n\
         Hint: Allowed: digits, d, +, -, kh, kl, dh, dl, ! and r, e.g. 4d6kh3+2"
    );
}

#[test]
fn test_explain_points_at_the_end_of_the_input() {
    let err = parse("2d6+").unwrap_err();
    assert_eq!(
        err.explain("2d6+"),
        "Expected a number or a dice roll, but the expression ends here\n\
         2d6+\n\
         \x20   ^\n\
         Hint: Write a number (3) or dice (1d6), e.g. 2d6+3"
    );
}

#[test]
fn test_explain_underlines_the_whole_token() {
    let err = parse("2d6kh5").unwrap_err();
    assert!(err.explain("2d6kh5").contains("\n2d6kh5\n     ^\n"));
    let err = parse("99999999999d6").unwrap_err();
    assert!(err.explain("99999999999d6").contains("\n^^^^^^^^^^^\n"));
}

#[test]
fn test_explain_without_position_is_the_plain_message() {
    let err = parse("").unwrap_err();
    assert_eq!(err.explain(""), err.to_string());
}

#[test]
fn test_roll_returns_the_same_errors() {
    let err = roll("2d6+").unwrap_err();
    assert!(matches!(err, DiceError::Syntax(_)));
}

#[test]
fn test_valid_expressions_still_parse() {
    for input in [
        "2d6+3",
        "4d6kh3",
        "2d20kl1",
        "1d6!",
        "1d6r1",
        " 2D6 + 1d8 - 2 ",
    ] {
        assert!(parse(input).is_ok(), "{input:?} should be valid");
    }
}
