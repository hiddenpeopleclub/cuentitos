//! Integer expression AST shared by the parser, runtime, and any future
//! consumer of `set`/`req` statements.
//!
//! Identifiers are resolved to [`crate::VariableId`]s at parse time; the AST
//! stored here is therefore self-contained and can be evaluated against a
//! variable-value lookup without re-resolving names.

use crate::VariableId;

/// Operator carried by a binary expression node.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BinOp {
    Add,
    Sub,
    Mul,
    Div,
}

/// Parsed integer expression with all identifiers resolved to declared
/// [`VariableId`]s.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Expr {
    Lit(i64),
    Var(VariableId),
    Binary {
        op: BinOp,
        left: Box<Expr>,
        right: Box<Expr>,
    },
}

/// Errors produced while evaluating a parsed expression at runtime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvalExprError {
    DivisionByZero,
    Overflow,
}

/// Evaluate `expr` against the supplied per-variable lookup function.
pub fn evaluate(expr: &Expr, lookup: &dyn Fn(VariableId) -> i64) -> Result<i64, EvalExprError> {
    match expr {
        Expr::Lit(n) => Ok(*n),
        Expr::Var(id) => Ok(lookup(*id)),
        Expr::Binary { op, left, right } => {
            let l = evaluate(left, lookup)?;
            let r = evaluate(right, lookup)?;
            apply_binop(*op, l, r)
        }
    }
}

/// Apply a binary operator with overflow + divide-by-zero checks.
pub fn apply_binop(op: BinOp, left: i64, right: i64) -> Result<i64, EvalExprError> {
    match op {
        BinOp::Add => left.checked_add(right).ok_or(EvalExprError::Overflow),
        BinOp::Sub => left.checked_sub(right).ok_or(EvalExprError::Overflow),
        BinOp::Mul => left.checked_mul(right).ok_or(EvalExprError::Overflow),
        BinOp::Div => {
            if right == 0 {
                Err(EvalExprError::DivisionByZero)
            } else {
                left.checked_div(right).ok_or(EvalExprError::Overflow)
            }
        }
    }
}
