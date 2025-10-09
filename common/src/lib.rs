pub mod block;
pub mod database;
pub mod path_resolver;
pub mod section;
pub mod test_case;

pub type StringId = usize;
pub type SectionId = usize;

// Re-export commonly used types
pub use block::{Block, BlockId, BlockType};
pub use database::Database;
pub use path_resolver::{PathResolutionError, PathResolver, ResolvedPath};
pub use section::Section;
