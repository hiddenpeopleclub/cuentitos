pub mod block;
pub mod boolean_expression;
pub mod database;
pub mod expression;
pub mod path_resolver;
pub mod requirement_statement;
pub mod section;
pub mod set_statement;
pub mod test_case;
pub mod value;
pub mod variable;

pub type StringId = usize;
pub type SectionId = usize;
pub type VariableId = usize;
pub type SetId = usize;
pub type RequirementId = usize;

// Re-export commonly used types
pub use block::{Block, BlockId, BlockType};
pub use boolean_expression::BooleanExpression;
pub use database::Database;
pub use expression::{evaluate, BinaryOperator, EvaluationError, Expression};
pub use path_resolver::{PathResolutionError, PathResolver, ResolvedPath};
pub use requirement_statement::{ComparisonOperator, RequirementStatement};
pub use section::Section;
pub use set_statement::{AssignmentOperator, SetStatement};
pub use value::{Value, ValueKind};
pub use variable::Variable;
