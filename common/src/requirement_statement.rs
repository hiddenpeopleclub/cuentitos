use crate::expression::Expression;
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

    /// Apply this comparison to two values of the same kind. Returns
    /// `None` if the operands have different kinds — parse-time type
    /// inference is expected to make that unreachable. Today only integer
    /// comparisons are implemented.
    #[must_use]
    pub fn apply(self, left: &Value, right: &Value) -> Option<bool> {
        match (left, right) {
            (Value::Integer(l), Value::Integer(r)) => Some(match self {
                ComparisonOperator::Equal => l == r,
                ComparisonOperator::NotEqual => l != r,
                ComparisonOperator::Less => l < r,
                ComparisonOperator::LessOrEqual => l <= r,
                ComparisonOperator::Greater => l > r,
                ComparisonOperator::GreaterOrEqual => l >= r,
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
