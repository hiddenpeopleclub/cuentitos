use crate::expression::Expression;
use crate::VariableId;

/// The assignment operator used by a `set` statement.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AssignmentOperator {
    Assign,
    AddAssign,
    SubtractAssign,
    MultiplyAssign,
    DivideAssign,
}

impl AssignmentOperator {
    /// True for compound operators that read the LHS before writing.
    /// Plain `Assign` overwrites unconditionally and never reads.
    #[must_use]
    pub fn is_compound(self) -> bool {
        !matches!(self, AssignmentOperator::Assign)
    }

    /// The source-syntax symbol for this operator (`=`, `+=`, `-=`, `*=`,
    /// `/=`). Used to echo the exact operator the author typed in diagnostics
    /// such as the compound-assignment-unsupported message.
    #[must_use]
    pub fn symbol(self) -> &'static str {
        match self {
            AssignmentOperator::Assign => "=",
            AssignmentOperator::AddAssign => "+=",
            AssignmentOperator::SubtractAssign => "-=",
            AssignmentOperator::MultiplyAssign => "*=",
            AssignmentOperator::DivideAssign => "/=",
        }
    }
}

/// Per-`set` statement metadata. Stored in `Database.sets`; referenced from a
/// [`crate::BlockType::Set`] block via its index.
///
/// `variable_id` is an *lvalue* — the assignment target — so it stays a bare
/// [`VariableId`] rather than an [`Expression`]. The expression on the right
/// is the *rvalue*, which is what gets evaluated.
///
/// Not `Eq`: `expression` may carry a `Value::Float` literal, whose `f64`
/// payload has no total equality.
#[derive(Debug, Clone, PartialEq)]
pub struct SetStatement {
    pub variable_id: VariableId,
    pub operator: AssignmentOperator,
    pub expression: Expression,
}

impl SetStatement {
    pub fn new(
        variable_id: VariableId,
        operator: AssignmentOperator,
        expression: Expression,
    ) -> Self {
        Self {
            variable_id,
            operator,
            expression,
        }
    }
}
