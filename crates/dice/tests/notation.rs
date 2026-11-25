/*
 ** EPITECH PROJECT, 2026
 ** notation.rs
 ** File description:
 ** Integration tests for dice notation parsing and rolling
 */

use dice::{DiceModifier, ExprTerm, Operator, parse, roll, roll_with_rng};
use rand::SeedableRng;
use rand::rngs::SmallRng;

fn seeded_rng() -> SmallRng {
    SmallRng::seed_from_u64(12345)
}

// Parsing tests

#[test]
fn test_parse_2d6() {
    let expr = parse("2d6").unwrap();
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
fn test_parse_2d6_plus_3() {
    let expr = parse("2d6+3").unwrap();
    assert_eq!(expr.terms.len(), 2);
    assert_eq!(expr.terms[1], (Operator::Add, ExprTerm::Constant(3)));
}

#[test]
fn test_parse_4d6kh3() {
    let expr = parse("4d6kh3").unwrap();
    match &expr.terms[0].1 {
        ExprTerm::Dice { modifier, .. } => {
            assert_eq!(*modifier, Some(DiceModifier::KeepHighest(3)));
        }
        _ => panic!("Expected dice term"),
    }
}

#[test]
fn test_parse_advantage() {
    let expr = parse("2d20kh1").unwrap();
    match &expr.terms[0].1 {
        ExprTerm::Dice {
            count,
            sides,
            modifier,
        } => {
            assert_eq!(*count, 2);
            assert_eq!(*sides, 20);
            assert_eq!(*modifier, Some(DiceModifier::KeepHighest(1)));
        }
        _ => panic!("Expected dice term"),
    }
}

#[test]
fn test_parse_disadvantage() {
    let expr = parse("2d20kl1").unwrap();
    match &expr.terms[0].1 {
        ExprTerm::Dice { modifier, .. } => {
            assert_eq!(*modifier, Some(DiceModifier::KeepLowest(1)));
        }
        _ => panic!("Expected dice term"),
    }
}

#[test]
fn test_parse_exploding() {
    let expr = parse("1d6!").unwrap();
    match &expr.terms[0].1 {
        ExprTerm::Dice { modifier, .. } => {
            assert_eq!(*modifier, Some(DiceModifier::Exploding));
        }
        _ => panic!("Expected dice term"),
    }
}

#[test]
fn test_parse_reroll() {
    let expr = parse("1d6r1").unwrap();
    match &expr.terms[0].1 {
        ExprTerm::Dice { modifier, .. } => {
            assert_eq!(*modifier, Some(DiceModifier::Reroll(1)));
        }
        _ => panic!("Expected dice term"),
    }
}

#[test]
fn test_parse_multi_dice() {
    let expr = parse("2d6+1d8+5").unwrap();
    assert_eq!(expr.terms.len(), 3);
}

#[test]
fn test_parse_subtraction() {
    let expr = parse("1d20-2").unwrap();
    assert_eq!(expr.terms[1], (Operator::Sub, ExprTerm::Constant(2)));
}

#[test]
fn test_parse_case_insensitive() {
    let expr = parse("4D6KH3").unwrap();
    match &expr.terms[0].1 {
        ExprTerm::Dice { modifier, .. } => {
            assert_eq!(*modifier, Some(DiceModifier::KeepHighest(3)));
        }
        _ => panic!("Expected dice term"),
    }
}

// Error cases

#[test]
fn test_parse_empty() {
    assert!(parse("").is_err());
}

#[test]
fn test_parse_zero_sides() {
    assert!(parse("1d0").is_err());
}

#[test]
fn test_parse_zero_dice() {
    assert!(parse("0d6").is_err());
}

#[test]
fn test_parse_invalid_chars() {
    assert!(parse("abc").is_err());
}

#[test]
fn test_parse_keep_more_than_rolled() {
    assert!(parse("2d6kh5").is_err());
}

// Rolling tests

#[test]
fn test_roll_simple_range() {
    let mut rng = seeded_rng();
    let result = roll_with_rng("1d6", &mut rng).unwrap();
    assert!(result.total >= 1 && result.total <= 6);
}

#[test]
fn test_roll_2d6_range() {
    let mut rng = seeded_rng();
    let result = roll_with_rng("2d6", &mut rng).unwrap();
    assert!(result.total >= 2 && result.total <= 12);
}

#[test]
fn test_roll_with_modifier_range() {
    let mut rng = seeded_rng();
    let result = roll_with_rng("2d6+3", &mut rng).unwrap();
    assert!(result.total >= 5 && result.total <= 15);
}

#[test]
fn test_roll_with_subtraction() {
    let mut rng = seeded_rng();
    let result = roll_with_rng("1d20-2", &mut rng).unwrap();
    assert!(result.total >= -1 && result.total <= 18);
}

#[test]
fn test_roll_keep_highest() {
    let mut rng = seeded_rng();
    let result = roll_with_rng("4d6kh3", &mut rng).unwrap();
    assert!(result.total >= 3 && result.total <= 18);
}

#[test]
fn test_roll_advantage() {
    let mut rng = seeded_rng();
    let result = roll_with_rng("2d20kh1", &mut rng).unwrap();
    assert!(result.total >= 1 && result.total <= 20);
}

#[test]
fn test_roll_disadvantage() {
    let mut rng = seeded_rng();
    let result = roll_with_rng("2d20kl1", &mut rng).unwrap();
    assert!(result.total >= 1 && result.total <= 20);
}

#[test]
fn test_roll_deterministic() {
    let mut rng1 = seeded_rng();
    let mut rng2 = seeded_rng();
    let r1 = roll_with_rng("4d6kh3+2d8+5", &mut rng1).unwrap();
    let r2 = roll_with_rng("4d6kh3+2d8+5", &mut rng2).unwrap();
    assert_eq!(r1.total, r2.total);
}

#[test]
fn test_roll_non_deterministic() {
    let result = roll("1d20").unwrap();
    assert!(result.total >= 1 && result.total <= 20);
}

#[test]
fn test_roll_exploding_d1_error() {
    let mut rng = seeded_rng();
    assert!(roll_with_rng("1d1!", &mut rng).is_err());
}

#[test]
fn test_roll_notation_preserved() {
    let mut rng = seeded_rng();
    let result = roll_with_rng("2d6+3", &mut rng).unwrap();
    assert_eq!(result.notation, "2d6+3");
}
