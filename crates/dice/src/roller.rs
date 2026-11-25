/*
 ** EPITECH PROJECT, 2026
 ** roller.rs
 ** File description:
 ** Evaluate a parsed dice expression by rolling dice
 */

use rand::{Rng, RngExt};
use serde::{Deserialize, Serialize};

use crate::ast::{DiceModifier, Expr, ExprTerm, Operator};
use crate::error::DiceError;

const MAX_EXPLOSIONS: u32 = 100;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiceGroupResult {
    pub rolls: Vec<u32>,
    pub kept: Vec<u32>,
    pub subtotal: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TermResult {
    Dice(DiceGroupResult),
    Constant(i64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RollResult {
    pub notation: String,
    pub details: Vec<(Operator, TermResult)>,
    pub total: i64,
}

pub fn evaluate(expr: &Expr, notation: &str, rng: &mut impl Rng) -> Result<RollResult, DiceError> {
    let mut total: i64 = 0;
    let mut details = Vec::new();

    for (op, term) in &expr.terms {
        let term_result = match term {
            ExprTerm::Dice {
                count,
                sides,
                modifier,
            } => {
                let group = roll_dice_group(*count, *sides, modifier.as_ref(), rng)?;
                let subtotal = group.subtotal;
                let result = TermResult::Dice(group);
                match op {
                    Operator::Add => total += subtotal,
                    Operator::Sub => total -= subtotal,
                }
                result
            }
            ExprTerm::Constant(val) => {
                match op {
                    Operator::Add => total += val,
                    Operator::Sub => total -= val,
                }
                TermResult::Constant(*val)
            }
        };
        details.push((*op, term_result));
    }

    Ok(RollResult {
        notation: notation.to_string(),
        details,
        total,
    })
}

fn roll_dice_group(
    count: u32,
    sides: u32,
    modifier: Option<&DiceModifier>,
    rng: &mut impl Rng,
) -> Result<DiceGroupResult, DiceError> {
    let mut rolls: Vec<u32> = Vec::new();

    for _ in 0..count {
        let mut value = rng.random_range(1..=sides);

        if let Some(DiceModifier::Reroll(threshold)) = modifier {
            if value <= *threshold {
                value = rng.random_range(1..=sides);
            }
        }
        if let Some(DiceModifier::Exploding) = modifier {
            if sides == 1 {
                return Err(DiceError::ExplosionLimit);
            }
            let mut explosion_count = 0;
            let mut exploded = value;
            while exploded == sides {
                explosion_count += 1;
                if explosion_count > MAX_EXPLOSIONS {
                    return Err(DiceError::ExplosionLimit);
                }
                let extra = rng.random_range(1..=sides);
                value += extra;
                exploded = extra;
            }
        }
        rolls.push(value);
    }
    let kept = apply_keep_drop(&rolls, modifier);
    let subtotal: i64 = kept.iter().map(|&v| i64::from(v)).sum();
    Ok(DiceGroupResult {
        rolls,
        kept,
        subtotal,
    })
}

fn apply_keep_drop(rolls: &[u32], modifier: Option<&DiceModifier>) -> Vec<u32> {
    let mut sorted = rolls.to_vec();
    sorted.sort_unstable();

    match modifier {
        Some(DiceModifier::KeepHighest(n)) => {
            let n = *n as usize;
            sorted.iter().rev().take(n).copied().collect()
        }
        Some(DiceModifier::KeepLowest(n)) => {
            let n = *n as usize;
            sorted.iter().take(n).copied().collect()
        }
        Some(DiceModifier::DropHighest(n)) => {
            let n = *n as usize;
            let keep = sorted.len().saturating_sub(n);
            sorted.iter().take(keep).copied().collect()
        }
        Some(DiceModifier::DropLowest(n)) => {
            let n = *n as usize;
            sorted.iter().skip(n).copied().collect()
        }
        _ => sorted,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::SeedableRng;
    use rand::rngs::SmallRng;

    fn seeded_rng() -> SmallRng {
        SmallRng::seed_from_u64(42)
    }

    #[test]
    fn test_simple_roll_range() {
        let mut rng = seeded_rng();
        let group = roll_dice_group(1, 6, None, &mut rng).unwrap();
        assert!(group.rolls[0] >= 1 && group.rolls[0] <= 6);
        assert_eq!(group.kept.len(), 1);
    }

    #[test]
    fn test_multiple_dice() {
        let mut rng = seeded_rng();
        let group = roll_dice_group(4, 6, None, &mut rng).unwrap();
        assert_eq!(group.rolls.len(), 4);
        assert_eq!(group.kept.len(), 4);
        for &v in &group.rolls {
            assert!(v >= 1 && v <= 6);
        }
    }

    #[test]
    fn test_keep_highest() {
        let rolls = vec![3, 1, 6, 4];
        let kept = apply_keep_drop(&rolls, Some(&DiceModifier::KeepHighest(3)));
        assert_eq!(kept.len(), 3);
        let sum: u32 = kept.iter().sum();
        assert_eq!(sum, 13); // 6 + 4 + 3
    }

    #[test]
    fn test_keep_lowest() {
        let rolls = vec![3, 1, 6, 4];
        let kept = apply_keep_drop(&rolls, Some(&DiceModifier::KeepLowest(1)));
        assert_eq!(kept, vec![1]);
    }

    #[test]
    fn test_drop_highest() {
        let rolls = vec![3, 1, 6, 4];
        let kept = apply_keep_drop(&rolls, Some(&DiceModifier::DropHighest(1)));
        assert_eq!(kept.len(), 3);
        assert!(!kept.contains(&6));
    }

    #[test]
    fn test_drop_lowest() {
        let rolls = vec![3, 1, 6, 4];
        let kept = apply_keep_drop(&rolls, Some(&DiceModifier::DropLowest(1)));
        assert_eq!(kept.len(), 3);
        assert!(!kept.contains(&1));
    }

    #[test]
    fn test_exploding_d1_error() {
        let mut rng = seeded_rng();
        let result = roll_dice_group(1, 1, Some(&DiceModifier::Exploding), &mut rng);
        assert!(result.is_err());
    }

    #[test]
    fn test_no_modifier() {
        let rolls = vec![2, 5, 3];
        let kept = apply_keep_drop(&rolls, None);
        assert_eq!(kept.len(), 3);
    }
}
