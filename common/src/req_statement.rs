use crate::expression::Expr;
use crate::VariableId;

/// The comparison operator used by a `req` statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompareOp {
    Eq,
    Ne,
    Gt,
    Lt,
    Ge,
    Le,
}

impl CompareOp {
    /// Apply this comparison to two integer operands.
    pub fn apply(self, left: i64, right: i64) -> bool {
        match self {
            CompareOp::Eq => left == right,
            CompareOp::Ne => left != right,
            CompareOp::Gt => left > right,
            CompareOp::Lt => left < right,
            CompareOp::Ge => left >= right,
            CompareOp::Le => left <= right,
        }
    }
}

/// Per-`req` statement metadata. Stored in `Database.req_statements`;
/// referenced from a [`crate::BlockType::Req`] block via its index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReqStatement {
    pub variable_id: VariableId,
    pub op: CompareOp,
    pub expression: Expr,
}

impl ReqStatement {
    pub fn new(variable_id: VariableId, op: CompareOp, expression: Expr) -> Self {
        Self {
            variable_id,
            op,
            expression,
        }
    }
}
