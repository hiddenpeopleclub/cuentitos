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
#[derive(Debug, Clone, PartialEq, Eq)]
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
    fn integer_equality_still_works() {
        let one = Value::Integer(1);
        let two = Value::Integer(2);
        assert_eq!(ComparisonOperator::Equal.apply(&one, &one), Ok(true));
        assert_eq!(ComparisonOperator::Less.apply(&one, &two), Ok(true));
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
}
