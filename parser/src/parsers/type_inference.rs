//! Parse-time type inference for [`Expression`]s.
//!
//! Walks an [`Expression`] AST and returns the [`ValueKind`] it produces, or
//! a [`TypeInferenceError`] when operands are inconsistent or an operator is
//! applied to an unsupported kind. Today every variable and arithmetic
//! operator is integer-typed, so the only way to trigger an error is by
//! constructing an `Expression` directly in tests; the inferrer is wired now
//! so adding `Boolean`/`Float`/`String` later is purely additive.

use cuentitos_common::{BinaryOperator, Database, Expression, Value, ValueKind};

/// Errors produced by [`infer_type`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TypeInferenceError {
    /// The two operands of a binary expression had incompatible kinds.
    Mismatch {
        operator: BinaryOperator,
        left: ValueKind,
        right: ValueKind,
    },
    /// A binary arithmetic operator was applied to a non-numeric kind.
    NonNumericArithmetic {
        operator: BinaryOperator,
        kind: ValueKind,
    },
}

/// Walk `expression` and return the [`ValueKind`] of the value it produces.
///
/// `database` is consulted to look up the declared kind of any
/// [`Expression::Variable`] reference.
pub fn infer_type(
    expression: &Expression,
    database: &Database,
) -> Result<ValueKind, TypeInferenceError> {
    match expression {
        Expression::Literal(value) => Ok(literal_kind(value)),
        Expression::Variable(id) => Ok(database.variables[*id].kind),
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            let left_kind = infer_type(left, database)?;
            let right_kind = infer_type(right, database)?;
            if left_kind != right_kind {
                return Err(TypeInferenceError::Mismatch {
                    operator: *operator,
                    left: left_kind,
                    right: right_kind,
                });
            }
            if !left_kind.is_numeric() {
                return Err(TypeInferenceError::NonNumericArithmetic {
                    operator: *operator,
                    kind: left_kind,
                });
            }
            Ok(left_kind)
        }
    }
}

fn literal_kind(value: &Value) -> ValueKind {
    value.kind()
}

#[cfg(test)]
mod tests {
    use super::*;
    use cuentitos_common::Variable;

    fn db_with(vars: &[&str]) -> Database {
        let mut db = Database::new();
        for name in vars {
            db.add_variable(Variable::new_integer(*name, 0));
        }
        db
    }

    #[test]
    fn integer_literal_infers_integer() {
        let db = Database::new();
        let expression = Expression::Literal(Value::Integer(7));
        assert_eq!(infer_type(&expression, &db).unwrap(), ValueKind::Integer);
    }

    #[test]
    fn integer_variable_infers_integer() {
        let db = db_with(&["x"]);
        let expression = Expression::Variable(0);
        assert_eq!(infer_type(&expression, &db).unwrap(), ValueKind::Integer);
    }

    #[test]
    fn integer_arithmetic_infers_integer() {
        let db = db_with(&["x"]);
        let expression = Expression::Binary {
            operator: BinaryOperator::Add,
            left: Box::new(Expression::Variable(0)),
            right: Box::new(Expression::Literal(Value::Integer(1))),
        };
        assert_eq!(infer_type(&expression, &db).unwrap(), ValueKind::Integer);
    }
}
