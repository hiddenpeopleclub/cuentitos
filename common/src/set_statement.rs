use crate::expression::Expr;
use crate::VariableId;

/// The assignment operator used by a `set` statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignOp {
    Assign,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
}

/// Per-set statement metadata. Stored in `Database.sets`; referenced from a
/// [`crate::BlockType::Set`] block via its index.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetStatement {
    pub variable_id: VariableId,
    pub op: AssignOp,
    pub expression: Expr,
}

impl SetStatement {
    pub fn new(variable_id: VariableId, op: AssignOp, expression: Expr) -> Self {
        Self {
            variable_id,
            op,
            expression,
        }
    }
}
