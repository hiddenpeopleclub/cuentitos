//! Polymorphic expression AST shared by the parser, runtime, and any future
//! consumer of `set`/`req` statements.
//!
//! Identifiers are resolved to [`crate::VariableId`]s at parse time; the AST
//! stored here is therefore self-contained and can be evaluated against a
//! variable-value lookup without re-resolving names.
//!
//! The AST and the evaluator are polymorphic over [`Value`] kinds. Today only
//! `Value::Integer` exists; adding `Boolean`/`Float`/`String` is additive —
//! new arms in [`BinaryOperator::apply`], new arms in [`evaluate`], no
//! restructuring at the call sites.

use std::borrow::Cow;

use crate::value::{Value, ValueKind};
use crate::VariableId;

/// Operator carried by a binary expression node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinaryOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
}

impl BinaryOperator {
    /// Apply this operator to two values. Both operands must share the same
    /// numeric kind: `(Integer, Integer)` folds with checked `i64` arithmetic,
    /// `(Float, Float)` folds with IEEE `f64` arithmetic. Mixed or non-numeric
    /// operands return [`EvaluationError::TypeMismatch`] — parse-time inference
    /// rejects those before the runtime ever folds them, so the arm only keeps
    /// the surface total.
    pub fn apply(self, left: &Value, right: &Value) -> Result<Value, EvaluationError> {
        match (self, left, right) {
            (BinaryOperator::Add, Value::Integer(l), Value::Integer(r)) => l
                .checked_add(*r)
                .map(Value::Integer)
                .ok_or(EvaluationError::Overflow),
            (BinaryOperator::Subtract, Value::Integer(l), Value::Integer(r)) => l
                .checked_sub(*r)
                .map(Value::Integer)
                .ok_or(EvaluationError::Overflow),
            (BinaryOperator::Multiply, Value::Integer(l), Value::Integer(r)) => l
                .checked_mul(*r)
                .map(Value::Integer)
                .ok_or(EvaluationError::Overflow),
            (BinaryOperator::Divide, Value::Integer(l), Value::Integer(r)) => {
                if *r == 0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    l.checked_div(*r)
                        .map(Value::Integer)
                        .ok_or(EvaluationError::Overflow)
                }
            }
            // Float arithmetic is IEEE: division does not truncate, and the
            // sign of zero is preserved (`0.0 * -1.0` -> `-0.0`). A result
            // that overflows to ±infinity is rejected as `FloatOverflow`
            // rather than stored, mirroring how float *defaults* reject
            // overflow at parse time; division by zero is rejected up front
            // rather than producing an IEEE infinity.
            (BinaryOperator::Add, Value::Float(l), Value::Float(r)) => finite_float(l + r),
            (BinaryOperator::Subtract, Value::Float(l), Value::Float(r)) => finite_float(l - r),
            (BinaryOperator::Multiply, Value::Float(l), Value::Float(r)) => finite_float(l * r),
            (BinaryOperator::Divide, Value::Float(l), Value::Float(r)) => {
                if *r == 0.0 {
                    Err(EvaluationError::DivisionByZero)
                } else {
                    finite_float(l / r)
                }
            }
            // Any remaining pairing — a `Boolean` operand or a mixed
            // `(Integer, Float)` — is a type error. Parse-time inference is
            // expected to reject these before the runtime ever folds them; the
            // arm keeps the surface total as `Value` grows new variants.
            (_, left, right) => Err(EvaluationError::TypeMismatch {
                expected: left.kind(),
                found: right.kind(),
            }),
        }
    }
}

/// Wrap a folded float result as a [`Value::Float`], rejecting a non-finite
/// outcome (overflow to ±infinity) as [`EvaluationError::FloatOverflow`]. NaN
/// cannot arise on the call paths here — operands are always finite and
/// division by zero is caught before the division runs — but it is also
/// non-finite and would be rejected the same way.
fn finite_float(value: f64) -> Result<Value, EvaluationError> {
    if value.is_finite() {
        Ok(Value::Float(value))
    } else {
        Err(EvaluationError::FloatOverflow)
    }
}

/// Parsed expression with all identifiers resolved to declared
/// [`VariableId`]s. Polymorphic over [`Value`] kinds.
///
/// Not `Eq`: a `Literal` may hold a `Value::Float`, whose `f64` payload has no
/// total equality. `PartialEq` is retained for tests and equality checks.
#[derive(Debug, Clone, PartialEq)]
pub enum Expression {
    Literal(Value),
    Variable(VariableId),
    Binary {
        operator: BinaryOperator,
        left: Box<Expression>,
        right: Box<Expression>,
    },
}

/// Errors produced while evaluating a parsed expression at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvaluationError {
    DivisionByZero,
    /// Integer arithmetic overflowed the `i64` range.
    Overflow,
    /// Float arithmetic overflowed the `f64` range to ±infinity. Distinct from
    /// [`Overflow`](Self::Overflow) so the runtime can surface a float-specific
    /// `RUNTIME ERROR: Float overflow.` message, parallel to the integer one.
    FloatOverflow,
    /// A binary operator was applied to operands of incompatible kinds. Today
    /// unreachable through normal parsing because parse-time inference rejects
    /// type-mismatched expressions; carried so the error surface is ready for
    /// future operators that take cross-kind operands.
    TypeMismatch {
        expected: ValueKind,
        found: ValueKind,
    },
}

/// Build the lookup closure expected by [`evaluate`] (and
/// [`crate::BooleanExpression::evaluate`]) from a flat slice of variable
/// values indexed by [`VariableId`]. Hides the explicit
/// `|id: VariableId| -> &Value { ... }` annotation that the closure
/// inference otherwise requires to drive the `'v` lifetime through.
pub fn variable_lookup<'v>(values: &'v [Value]) -> impl Fn(VariableId) -> &'v Value + 'v {
    move |id: VariableId| &values[id]
}

/// Evaluate `expression` against the supplied per-variable lookup. The
/// lookup returns a borrow of the current [`Value`] for a [`VariableId`];
/// the result is a [`Cow`] so that variable references and literals don't
/// allocate, and only arithmetic combinations produce owned values. With
/// `Value::Integer` (today's only variant) the cost is the same either way,
/// but for future non-`Copy` variants (`String`, etc.) this avoids
/// per-lookup allocation.
pub fn evaluate<'v>(
    expression: &'v Expression,
    lookup: &dyn Fn(VariableId) -> &'v Value,
) -> Result<Cow<'v, Value>, EvaluationError> {
    match expression {
        Expression::Literal(value) => Ok(Cow::Borrowed(value)),
        Expression::Variable(id) => Ok(Cow::Borrowed(lookup(*id))),
        Expression::Binary {
            operator,
            left,
            right,
        } => {
            let left_value = evaluate(left, lookup)?;
            let right_value = evaluate(right, lookup)?;
            operator.apply(&left_value, &right_value).map(Cow::Owned)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn float(value: f64) -> Value {
        Value::Float(value)
    }

    #[test]
    fn float_arithmetic_folds_with_ieee_semantics() {
        assert_eq!(
            BinaryOperator::Add.apply(&float(5.0), &float(3.0)).unwrap(),
            float(8.0)
        );
        assert_eq!(
            BinaryOperator::Subtract
                .apply(&float(5.0), &float(3.0))
                .unwrap(),
            float(2.0)
        );
        assert_eq!(
            BinaryOperator::Multiply
                .apply(&float(5.0), &float(3.0))
                .unwrap(),
            float(15.0)
        );
    }

    #[test]
    fn float_division_does_not_truncate() {
        // Unlike integer division, `7.0 / 2.0` keeps the fractional part.
        assert_eq!(
            BinaryOperator::Divide
                .apply(&float(7.0), &float(2.0))
                .unwrap(),
            float(3.5)
        );
        assert_eq!(
            BinaryOperator::Divide
                .apply(&float(1.0), &float(4.0))
                .unwrap(),
            float(0.25)
        );
    }

    #[test]
    fn float_division_by_zero_is_an_error_not_infinity() {
        assert_eq!(
            BinaryOperator::Divide
                .apply(&float(10.0), &float(0.0))
                .unwrap_err(),
            EvaluationError::DivisionByZero
        );
    }

    #[test]
    fn float_overflow_to_infinity_is_rejected() {
        assert_eq!(
            BinaryOperator::Multiply
                .apply(&float(1e200), &float(1e200))
                .unwrap_err(),
            EvaluationError::FloatOverflow
        );
    }

    #[test]
    fn float_multiplication_preserves_negative_zero() {
        // IEEE distinguishes `+0.0` from `-0.0`; `0.0 * -1.0` is `-0.0`.
        let result = BinaryOperator::Multiply
            .apply(&float(0.0), &float(-1.0))
            .unwrap();
        let Value::Float(value) = result else {
            panic!("expected float, got {result:?}");
        };
        assert_eq!(value, 0.0);
        assert!(value.is_sign_negative(), "expected -0.0, got {value}");
    }

    #[test]
    fn mixed_int_and_float_operands_are_a_type_error() {
        // Parse-time inference rejects these, but the runtime arm must stay
        // total: a mixed pairing surfaces as a type mismatch, not a panic.
        assert_eq!(
            BinaryOperator::Add
                .apply(&Value::Integer(1), &float(1.0))
                .unwrap_err(),
            EvaluationError::TypeMismatch {
                expected: ValueKind::Integer,
                found: ValueKind::Float,
            }
        );
    }
}
