//! Boolean-expression tree carried by [`crate::BlockType::Requirement`].
//!
//! A `req` condition is a tree of comparisons combined by the lowercase
//! logical operators `and`, `or`, and `not`. Leaves are
//! [`RequirementStatement`]s — a single comparison such as `health > 0`.
//!
//! The tree shape mirrors the grammar precedence: `not` binds tightest,
//! then `and`, then `or`. Combinators short-circuit at evaluation time;
//! see [`BooleanExpression::evaluate`].

use crate::expression::{evaluate as evaluate_expression, EvaluationError, Expression};
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
                // Reading an unset enum operand is a runtime error — enums have
                // no default, so a comparison against one before its first
                // `set` cannot be evaluated. Checked here (rather than in
                // `ComparisonOperator::apply`) so the offending variable's id
                // is still in hand to name in the diagnostic. Short-circuiting
                // means an unset enum in a branch the result doesn't depend on
                // never fires.
                if let Some(variable) = unset_enum_operand(&statement.left, &left)
                    .or_else(|| unset_enum_operand(&statement.right, &right))
                {
                    return Err(EvaluationError::UnsetEnum { variable });
                }
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

/// If `expression` is a variable reference whose current `value` is an unset
/// enum, return that variable's id; otherwise `None`. An unset enum can only
/// reach a comparison through a variable read — literals are always assigned —
/// so the variable form is the only one worth inspecting.
fn unset_enum_operand(expression: &Expression, value: &Value) -> Option<VariableId> {
    match (expression, value) {
        (Expression::Variable(id), Value::EnumUnset { .. }) => Some(*id),
        _ => None,
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

    fn enum_value(variants: &[&str], value: &str) -> Value {
        Value::Enum {
            variants: variants.iter().map(|v| v.to_string()).collect(),
            value: value.to_string(),
        }
    }

    fn enum_unset(variants: &[&str]) -> Value {
        Value::EnumUnset {
            variants: variants.iter().map(|v| v.to_string()).collect(),
        }
    }

    /// `req <var0> = <variant>` against a one-variable lookup.
    fn enum_var_eq_variant(variant: Value) -> BooleanExpression {
        BooleanExpression::Comparison(RequirementStatement::new(
            Expression::Variable(0),
            ComparisonOperator::Equal,
            Expression::Literal(variant),
        ))
    }

    #[test]
    fn assigned_enum_compares_by_selected_variant() {
        // `evaluate` ties the expression and the lookup's borrow under one
        // lifetime, so the expressions must be declared before `lookup`.
        let matches = enum_var_eq_variant(enum_value(&["happy", "sad"], "happy"));
        let differs = enum_var_eq_variant(enum_value(&["happy", "sad"], "sad"));
        let values = [enum_value(&["happy", "sad"], "happy")];
        let lookup = crate::expression::variable_lookup(&values);
        assert_eq!(matches.evaluate(&lookup), Ok(true));
        assert_eq!(differs.evaluate(&lookup), Ok(false));
    }

    #[test]
    fn reading_an_unset_enum_in_a_comparison_is_an_error() {
        let expression = enum_var_eq_variant(enum_value(&["happy", "sad"], "happy"));
        let values = [enum_unset(&["happy", "sad"])];
        let lookup = crate::expression::variable_lookup(&values);
        assert_eq!(
            expression.evaluate(&lookup),
            Err(EvaluationError::UnsetEnum { variable: 0 })
        );
    }

    #[test]
    fn unset_enum_in_short_circuited_branch_does_not_error() {
        // `false and (mood = happy)` with `mood` unset: the right branch is
        // never reached, so the unset read never fires.
        let expression = BooleanExpression::And(
            Box::new(comparison(0, ComparisonOperator::Greater, 1)),
            Box::new(enum_var_eq_variant(enum_value(&["happy", "sad"], "happy"))),
        );
        let values = [enum_unset(&["happy", "sad"])];
        let lookup = crate::expression::variable_lookup(&values);
        assert_eq!(expression.evaluate(&lookup), Ok(false));
    }
}
