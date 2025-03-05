pub mod block;
pub mod database;
pub mod test_case;

pub type StringId = usize;

// Re-export commonly used types
pub use block::{Block, BlockId, BlockType};
pub use database::Database;
