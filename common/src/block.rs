/// A unique identifier for a block in the database.
pub type BlockId = usize;

/// The type of a block in the cuentitos script.
#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    /// The special START block that begins every script
    Start,
    /// The special END block that ends every script
    End,
    /// A text block, with an index into the strings table
    String(usize),
    /// A section header block, with an index into the strings table for the title
    Section(usize),
}

/// A block in the cuentitos script.
///
/// Blocks are the fundamental units of a cuentitos script. They can be:
/// - Special blocks (START, END)
/// - Text blocks with content
/// - Section headers that organize content
///
/// Blocks form a tree structure through parent-child relationships,
/// and maintain their indentation level for proper nesting.
#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    /// The type of this block and its associated data
    pub block_type: BlockType,
    /// The ID of this block's parent, if any
    pub parent_id: Option<BlockId>,
    /// The IDs of this block's children
    pub children: Vec<BlockId>,
    /// The indentation level of this block
    pub level: usize,
}

impl Block {
    /// Creates a new block with the given type, parent, and level.
    ///
    /// # Arguments
    ///
    /// * `block_type` - The type of block to create
    /// * `parent_id` - The ID of the parent block, if any
    /// * `level` - The indentation level of the block
    ///
    /// # Returns
    ///
    /// A new Block instance with no children
    pub fn new(block_type: BlockType, parent_id: Option<BlockId>, level: usize) -> Self {
        Self {
            block_type,
            parent_id,
            children: Vec::new(),
            level,
        }
    }

    /// Adds a child block to this block.
    ///
    /// # Arguments
    ///
    /// * `child_id` - The ID of the child block to add
    pub fn add_child(&mut self, child_id: BlockId) {
        self.children.push(child_id);
    }

    /// Returns whether this block is a leaf node (has no children).
    ///
    /// # Returns
    ///
    /// `true` if the block has no children, `false` otherwise
    pub fn is_leaf(&self) -> bool {
        self.children.is_empty()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_block_creation() {
        let block = Block::new(BlockType::Start, None, 0);
        assert_eq!(block.parent_id, None);
        assert!(block.children.is_empty());
        assert_eq!(block.level, 0);
    }

    #[test]
    fn test_block_with_parent() {
        let block = Block::new(BlockType::String(0), Some(1), 1);
        assert_eq!(block.parent_id, Some(1));
        assert!(block.children.is_empty());
        assert_eq!(block.level, 1);
    }

    #[test]
    fn test_add_child() {
        let mut block = Block::new(BlockType::String(0), None, 0);
        block.add_child(1);
        assert_eq!(block.children, vec![1]);
        assert!(!block.is_leaf());
    }

    #[test]
    fn test_is_leaf() {
        let block = Block::new(BlockType::String(0), None, 0);
        assert!(block.is_leaf());
    }
}
