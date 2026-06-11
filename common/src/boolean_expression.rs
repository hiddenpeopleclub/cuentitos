//! Boolean-expression tree carried by [`crate::BlockType::Requirement`].
//!
//! A `req` condition is a tree of comparisons combined by the lowercase
//! logical operators `and`, `or`, and `not`. Leaves are
//! [`RequirementStatement`]s — a single comparison such as `health > 0`.
//!
//! The tree shape mirrors the grammar precedence: `not` binds tightest,
//! then `and`, then `or`. Combinators short-circuit at evaluation time;
//! see [`BooleanExpression::evaluate`].

use crate::expression::{evaluate as evaluate_expression, EvaluationError};
use crate::requirement_statement::RequirementStatement;
use crate::value::Value;
use crate::VariableId;

/// A `req` condition. Either a single comparison leaf or a logical
/// combination of sub-expressions.
///
/// Not `Eq`: leaves carry [`RequirementStatement`]s whose expressions may hold
/// `Value::Float` literals, and `f64` has no total equality.
#[derive(Debug, Clone, PartialEq)]
pub enum BooleanExpression {
    Comparison(RequirementStatement),
    And(Box<BooleanExpression>, Box<BooleanExpression>),
    Or(Box<BooleanExpression>, Box<BooleanExpression>),
    Not(Box<BooleanExpression>),
}

impl BooleanExpression {
    /// Evaluate the tree to a boolean. Short-circuits left-to-right, so
    /// runtime errors in branches the result doesn't depend on are
    /// skipped (e.g. divide-by-zero on the right of an `and` whose left
    /// is already false never fires).
    pub fn evaluate<'v>(
        &'v self,
        lookup: &dyn Fn(VariableId) -> &'v Value,
    ) -> Result<bool, EvaluationError> {
        match self {
            BooleanExpression::Comparison(statement) => {
                let left = evaluate_expression(&statement.left, lookup)?;
                let right = evaluate_expression(&statement.right, lookup)?;
                statement.operator.apply(&left, &right)
            }
            BooleanExpression::And(left, right) => {
                if !left.evaluate(lookup)? {
                    return Ok(false);
                }
                right.evaluate(lookup)
            }
            BooleanExpression::Or(left, right) => {
                if left.evaluate(lookup)? {
                    return Ok(true);
                }
                right.evaluate(lookup)
            }
            BooleanExpression::Not(inner) => Ok(!inner.evaluate(lookup)?),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::expression::Expression;
    use crate::requirement_statement::ComparisonOperator;

    fn comparison(left: i64, op: ComparisonOperator, right: i64) -> BooleanExpression {
        BooleanExpression::Comparison(RequirementStatement::new(
            Expression::Literal(Value::Integer(left)),
            op,
            Expression::Literal(Value::Integer(right)),
        ))
    }

    fn no_vars<'a>() -> impl Fn(VariableId) -> &'a Value {
        |_| panic!("no variables expected")
    }

    #[test]
    fn evaluates_single_comparison() {
        let expression = comparison(5, ComparisonOperator::Greater, 0);
        assert_eq!(expression.evaluate(&no_vars()), Ok(true));
    }

    #[test]
    fn evaluates_and_true() {
        let expression = BooleanExpression::And(
            Box::new(comparison(5, ComparisonOperator::Greater, 0)),
            Box::new(comparison(5, ComparisonOperator::Less, 10)),
        );
        assert_eq!(expression.evaluate(&no_vars()), Ok(true));
    }

    #[test]
    fn evaluates_and_short_circuits_on_false_left() {
        // The right side would divide by zero if evaluated; short-circuit
        // means it is never reached.
        let right_with_division = BooleanExpression::Comparison(RequirementStatement::new(
            Expression::Binary {
                operator: crate::expression::BinaryOperator::Divide,
                left: Box::new(Expression::Literal(Value::Integer(1))),
                right: Box::new(Expression::Literal(Value::Integer(0))),
            },
            ComparisonOperator::Greater,
            Expression::Literal(Value::Integer(0)),
        ));
        let expression = BooleanExpression::And(
            Box::new(comparison(0, ComparisonOperator::Greater, 1)),
            Box::new(right_with_division),
        );
        assert_eq!(expression.evaluate(&no_vars()), Ok(false));
    }

    #[test]
    fn evaluates_or_short_circuits_on_true_left() {
        let right_with_division = BooleanExpression::Comparison(RequirementStatement::new(
            Expression::Binary {
                operator: crate::expression::BinaryOperator::Divide,
                left: Box::new(Expression::Literal(Value::Integer(1))),
                right: Box::new(Expression::Literal(Value::Integer(0))),
            },
            ComparisonOperator::Greater,
            Expression::Literal(Value::Integer(0)),
        ));
        let expression = BooleanExpression::Or(
            Box::new(comparison(5, ComparisonOperator::Greater, 0)),
            Box::new(right_with_division),
        );
        assert_eq!(expression.evaluate(&no_vars()), Ok(true));
    }

    #[test]
    fn evaluates_not() {
        let expression =
            BooleanExpression::Not(Box::new(comparison(5, ComparisonOperator::Greater, 0)));
        assert_eq!(expression.evaluate(&no_vars()), Ok(false));

        let expression =
            BooleanExpression::Not(Box::new(comparison(0, ComparisonOperator::Greater, 5)));
        assert_eq!(expression.evaluate(&no_vars()), Ok(true));
    }

    #[test]
    fn evaluates_combination_with_precedence() {
        // (a > 0) or ((b > 0) and (c > 0))
        let expression = BooleanExpression::Or(
            Box::new(comparison(1, ComparisonOperator::Greater, 0)),
            Box::new(BooleanExpression::And(
                Box::new(comparison(0, ComparisonOperator::Greater, 0)),
                Box::new(comparison(0, ComparisonOperator::Greater, 0)),
            )),
        );
        assert_eq!(expression.evaluate(&no_vars()), Ok(true));
    }
}
