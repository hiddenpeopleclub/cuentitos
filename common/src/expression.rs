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
    /// Apply this operator to two values. Today every operator requires both
    /// operands to be `Integer`; mixed or non-numeric operands return
    /// [`EvaluationError::TypeMismatch`]. Adding `Float` later means adding
    /// a `(Float, Float)` arm and a numeric-promotion rule.
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
        }
    }
}

/// Parsed expression with all identifiers resolved to declared
/// [`VariableId`]s. Polymorphic over [`Value`] kinds.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    Overflow,
    /// A binary operator was applied to operands of incompatible kinds. Today
    /// unreachable through normal parsing because parse-time inference rejects
    /// type-mismatched expressions; carried so the error surface is ready for
    /// future operators that take cross-kind operands.
    TypeMismatch {
        expected: ValueKind,
        found: ValueKind,
    },
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
