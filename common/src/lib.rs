pub mod block;
pub mod database;
pub mod expression;
pub mod path_resolver;
pub mod req_statement;
pub mod section;
pub mod set_statement;
pub mod test_case;
pub mod variable;

pub type StringId = usize;
pub type SectionId = usize;
pub type VariableId = usize;
pub type SetId = usize;
pub type ReqId = usize;

// Re-export commonly used types
pub use block::{Block, BlockId, BlockType};
pub use database::Database;
pub use expression::{apply_binop, evaluate, BinOp, EvalExprError, Expr};
pub use path_resolver::{PathResolutionError, PathResolver, ResolvedPath};
pub use req_statement::{CompareOp, ReqStatement};
pub use section::Section;
pub use set_statement::{AssignOp, SetStatement};
pub use variable::{Variable, VariableKind, VariableValue};
