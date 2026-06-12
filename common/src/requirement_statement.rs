use crate::expression::{EvaluationError, Expression};
use crate::value::Value;

/// The comparison operator used by a `req` statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ComparisonOperator {
    Equal,
    NotEqual,
    Less,
    LessOrEqual,
    Greater,
    GreaterOrEqual,
}

impl ComparisonOperator {
    /// True for the four ordering operators (`< <= > >=`). Equality
    /// (`= !=`) works on any kind that supports `PartialEq`; ordering
    /// requires the kind to additionally be totally ordered.
    #[must_use]
    pub fn requires_ordering(self) -> bool {
        matches!(
            self,
            ComparisonOperator::Less
                | ComparisonOperator::LessOrEqual
                | ComparisonOperator::Greater
                | ComparisonOperator::GreaterOrEqual
        )
    }

    /// The source-syntax spelling of this operator (`=`, `!=`, `<`, `<=`,
    /// `>`, `>=`). Used by diagnostics that echo the operator the author
    /// typed — e.g. the bool ordering-operator rejection.
    #[must_use]
    pub fn symbol(self) -> &'static str {
        match self {
            ComparisonOperator::Equal => "=",
            ComparisonOperator::NotEqual => "!=",
            ComparisonOperator::Less => "<",
            ComparisonOperator::LessOrEqual => "<=",
            ComparisonOperator::Greater => ">",
            ComparisonOperator::GreaterOrEqual => ">=",
        }
    }

    /// Apply this comparison to two values.
    ///
    /// Integers support the full operator set. Booleans support only equality
    /// (`=`/`!=`); ordering operators on booleans are rejected at parse time
    /// (see the `NonOrderedComparison` guard in the requirement parser) and so
    /// never reach this method. Operands of differing kinds return
    /// [`EvaluationError::TypeMismatch`] — parse-time inference is expected to
    /// make that unreachable too, but the arm keeps the match total.
    pub fn apply(self, left: &Value, right: &Value) -> Result<bool, EvaluationError> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Ok(match self {
                ComparisonOperator::Equal => l == r,
                ComparisonOperator::NotEqual => l != r,
                ComparisonOperator::Less => l < r,
                ComparisonOperator::LessOrEqual => l <= r,
                ComparisonOperator::Greater => l > r,
                ComparisonOperator::GreaterOrEqual => l >= r,
            }),
            (Value::Float(l), Value::Float(r)) => Ok(match self {
                // IEEE-754 comparison semantics, straight from `f64`'s
                // `PartialOrd`/`PartialEq`: `-0.0 == 0.0` is true, ordering
                // against `-0.0` ignores the sign, and equality is the exact
                // bit-faithful comparison of the stored values (so
                // `0.1 + 0.2 != 0.3`). NaN never reaches here — it can't be
                // produced by the float arithmetic the parser admits.
                ComparisonOperator::Equal => l == r,
                ComparisonOperator::NotEqual => l != r,
                ComparisonOperator::Less => l < r,
                ComparisonOperator::LessOrEqual => l <= r,
                ComparisonOperator::Greater => l > r,
                ComparisonOperator::GreaterOrEqual => l >= r,
            }),
            (Value::Boolean(l), Value::Boolean(r)) => match self {
                ComparisonOperator::Equal => Ok(l == r),
                ComparisonOperator::NotEqual => Ok(l != r),
                // Ordering two booleans is meaningless and is rejected at parse
                // time, so this is structurally unreachable through the normal
                // pipeline.
                ComparisonOperator::Less
                | ComparisonOperator::LessOrEqual
                | ComparisonOperator::Greater
                | ComparisonOperator::GreaterOrEqual => {
                    unreachable!("ordering comparison on booleans is rejected at parse time")
                }
            },
            (Value::String(l), Value::String(r)) => match self {
                // Exact, case- and whitespace-sensitive comparison straight
                // from `String`'s `PartialEq` — escape sequences were decoded
                // at parse time, so this compares the resolved character
                // content.
                ComparisonOperator::Equal => Ok(l == r),
                ComparisonOperator::NotEqual => Ok(l != r),
                // Strings have no ordering; the four ordering operators are
                // rejected at parse time, so this is structurally unreachable
                // through the normal pipeline.
                ComparisonOperator::Less
                | ComparisonOperator::LessOrEqual
                | ComparisonOperator::Greater
                | ComparisonOperator::GreaterOrEqual => {
                    unreachable!("ordering comparison on strings is rejected at parse time")
                }
            },
            // Two enums compare by their selected variant name. Either side may
            // be assigned (`Enum`) or the `EnumUnset` sentinel; an unset enum
            // equals nothing — not a real variant, and not another unset enum —
            // so any unset operand makes `=` false and `!=` true. Only equality
            // is meaningful (enums have no ordering); the four ordering
            // operators are rejected at parse time, so they're structurally
            // unreachable here.
            //
            // On the live `req` path an unset operand never actually reaches
            // this arm: reading an unset enum is caught upstream in
            // [`crate::BooleanExpression::evaluate`] and surfaced as the named
            // [`EvaluationError::UnsetEnum`]. The sentinel handling here keeps
            // `apply` a total, sensible comparison primitive for any direct
            // caller that bypasses that policy layer.
            (
                left_enum @ (Value::Enum { .. } | Value::EnumUnset { .. }),
                right_enum @ (Value::Enum { .. } | Value::EnumUnset { .. }),
            ) => {
                let selected_variant = |value: &Value| match value {
                    Value::Enum { value, .. } => Some(value.clone()),
                    _ => None,
                };
                let equal = matches!(
                    (selected_variant(left_enum), selected_variant(right_enum)),
                    (Some(left_value), Some(right_value)) if left_value == right_value
                );
                match self {
                    ComparisonOperator::Equal => Ok(equal),
                    ComparisonOperator::NotEqual => Ok(!equal),
                    ComparisonOperator::Less
                    | ComparisonOperator::LessOrEqual
                    | ComparisonOperator::Greater
                    | ComparisonOperator::GreaterOrEqual => {
                        unreachable!("ordering comparison on enums is rejected at parse time")
                    }
                }
            }
            // Operands of differing kinds (e.g. an `Integer` against a
            // `Boolean`) are a type error. Parse-time inference is expected to
            // make this unreachable; the arm keeps the match total as `Value`
            // grows new variants.
            (left, right) => Err(EvaluationError::TypeMismatch {
                expected: left.kind(),
                found: right.kind(),
            }),
        }
    }
}

/// Per-`req` statement metadata. Stored in `Database.requirements`;
/// referenced from a [`crate::BlockType::Requirement`] block via its index.
///
/// The data model is symmetric — both sides are full [`Expression`]s. The
/// current parser only accepts `<var> <op> <expr>` (and wraps the LHS as
/// [`Expression::Variable`]), but relaxing the grammar to accept arbitrary
/// expressions on both sides is purely a parser change.
///
/// Not `Eq`: the `left`/`right` expressions may carry `Value::Float` literals,
/// whose `f64` payload has no total equality.
#[derive(Debug, Clone, PartialEq)]
pub struct RequirementStatement {
    pub left: Expression,
    pub operator: ComparisonOperator,
    pub right: Expression,
}

impl RequirementStatement {
    pub fn new(left: Expression, operator: ComparisonOperator, right: Expression) -> Self {
        Self {
            left,
            operator,
            right,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::value::ValueKind;

    #[test]
    fn boolean_equality_compares_by_value() {
        let t = Value::Boolean(true);
        let f = Value::Boolean(false);
        assert_eq!(ComparisonOperator::Equal.apply(&t, &t), Ok(true));
        assert_eq!(ComparisonOperator::Equal.apply(&t, &f), Ok(false));
        assert_eq!(ComparisonOperator::NotEqual.apply(&t, &f), Ok(true));
        assert_eq!(ComparisonOperator::NotEqual.apply(&t, &t), Ok(false));
    }

    #[test]
    fn float_comparisons_follow_ieee_semantics() {
        let one_five = Value::Float(1.5);
        let two_five = Value::Float(2.5);
        assert_eq!(
            ComparisonOperator::Equal.apply(&one_five, &one_five),
            Ok(true)
        );
        assert_eq!(
            ComparisonOperator::NotEqual.apply(&one_five, &two_five),
            Ok(true)
        );
        assert_eq!(
            ComparisonOperator::Less.apply(&one_five, &two_five),
            Ok(true)
        );
        assert_eq!(
            ComparisonOperator::LessOrEqual.apply(&one_five, &one_five),
            Ok(true)
        );
        assert_eq!(
            ComparisonOperator::Greater.apply(&two_five, &one_five),
            Ok(true)
        );
        assert_eq!(
            ComparisonOperator::GreaterOrEqual.apply(&one_five, &one_five),
            Ok(true)
        );
    }

    #[test]
    fn negative_zero_compares_equal_to_positive_zero() {
        // IEEE-754: `-0.0 == 0.0` is true and `-0.0 < 0.0` is false, even
        // though the two have distinct bit patterns.
        let neg_zero = Value::Float(-0.0);
        let pos_zero = Value::Float(0.0);
        assert_eq!(
            ComparisonOperator::Equal.apply(&neg_zero, &pos_zero),
            Ok(true)
        );
        assert_eq!(
            ComparisonOperator::Less.apply(&neg_zero, &pos_zero),
            Ok(false)
        );
    }

    #[test]
    fn inexact_sum_is_not_equal_to_nearest_literal() {
        // `0.1 + 0.2` stores `0.30000000000000004`, fractionally larger than
        // the nearest `f64` to `0.3`, so equality fails and `>` passes.
        let sum = Value::Float(0.1 + 0.2);
        let three_tenths = Value::Float(0.3);
        assert_eq!(
            ComparisonOperator::Equal.apply(&sum, &three_tenths),
            Ok(false)
        );
        assert_eq!(
            ComparisonOperator::Greater.apply(&sum, &three_tenths),
            Ok(true)
        );
    }

    #[test]
    fn integer_equality_still_works() {
        let one = Value::Integer(1);
        let two = Value::Integer(2);
        assert_eq!(ComparisonOperator::Equal.apply(&one, &one), Ok(true));
        assert_eq!(ComparisonOperator::Less.apply(&one, &two), Ok(true));
    }

    #[test]
    fn string_equality_compares_by_value() {
        let aria = Value::String("Aria".to_string());
        let brenn = Value::String("Brenn".to_string());
        assert_eq!(ComparisonOperator::Equal.apply(&aria, &aria), Ok(true));
        assert_eq!(ComparisonOperator::Equal.apply(&aria, &brenn), Ok(false));
        assert_eq!(ComparisonOperator::NotEqual.apply(&aria, &brenn), Ok(true));
        assert_eq!(ComparisonOperator::NotEqual.apply(&aria, &aria), Ok(false));
    }

    #[test]
    fn string_equality_is_case_and_whitespace_sensitive() {
        let aria = Value::String("Aria".to_string());
        let lower = Value::String("aria".to_string());
        let trailing = Value::String("Aria ".to_string());
        assert_eq!(ComparisonOperator::Equal.apply(&aria, &lower), Ok(false));
        assert_eq!(ComparisonOperator::Equal.apply(&aria, &trailing), Ok(false));
        assert_eq!(ComparisonOperator::NotEqual.apply(&aria, &lower), Ok(true));
    }

    #[test]
    fn mixed_kinds_are_a_type_mismatch() {
        let int = Value::Integer(1);
        let boolean = Value::Boolean(true);
        assert_eq!(
            ComparisonOperator::Equal.apply(&int, &boolean),
            Err(EvaluationError::TypeMismatch {
                expected: ValueKind::Integer,
                found: ValueKind::Boolean,
            })
        );
    }

    fn enum_value(value: &str) -> Value {
        Value::Enum {
            variants: vec!["happy".to_string(), "sad".to_string()],
            value: value.to_string(),
        }
    }

    fn enum_unset() -> Value {
        Value::EnumUnset {
            variants: vec!["happy".to_string(), "sad".to_string()],
        }
    }

    #[test]
    fn assigned_enums_compare_by_selected_variant() {
        let happy = enum_value("happy");
        let sad = enum_value("sad");
        assert_eq!(ComparisonOperator::Equal.apply(&happy, &happy), Ok(true));
        assert_eq!(ComparisonOperator::Equal.apply(&happy, &sad), Ok(false));
        assert_eq!(ComparisonOperator::NotEqual.apply(&happy, &sad), Ok(true));
        assert_eq!(
            ComparisonOperator::NotEqual.apply(&happy, &happy),
            Ok(false)
        );
    }

    #[test]
    fn unset_enum_is_a_sentinel_that_equals_nothing() {
        // An unset operand on either side (or both) never equals: `=` is false
        // and `!=` is true. This is `apply`'s total fallback — the live `req`
        // path errors on an unset read before reaching here.
        let happy = enum_value("happy");
        let unset = enum_unset();
        assert_eq!(ComparisonOperator::Equal.apply(&unset, &happy), Ok(false));
        assert_eq!(ComparisonOperator::Equal.apply(&happy, &unset), Ok(false));
        assert_eq!(ComparisonOperator::Equal.apply(&unset, &unset), Ok(false));
        assert_eq!(ComparisonOperator::NotEqual.apply(&unset, &happy), Ok(true));
        assert_eq!(ComparisonOperator::NotEqual.apply(&happy, &unset), Ok(true));
        assert_eq!(ComparisonOperator::NotEqual.apply(&unset, &unset), Ok(true));
    }

    #[test]
    fn unset_enum_against_a_non_enum_is_still_a_type_mismatch() {
        // The enum arm is scoped to enum-vs-enum, so an unset enum paired with
        // a non-enum still falls through to the type-mismatch catch-all rather
        // than being silently swallowed by the sentinel handling.
        let unset = enum_unset();
        let int = Value::Integer(1);
        assert_eq!(
            ComparisonOperator::Equal.apply(&unset, &int),
            Err(EvaluationError::TypeMismatch {
                expected: ValueKind::Enum,
                found: ValueKind::Integer,
            })
        );
    }
}
