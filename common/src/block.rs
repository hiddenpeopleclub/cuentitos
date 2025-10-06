use crate::StringId;

pub type BlockId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Start,
    String(StringId),
    Section { id: String, display_name: String },
    GoToSection { path: String, target_block_id: BlockId },
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub block_type: BlockType,
    pub parent_id: Option<BlockId>,
    pub children: Vec<BlockId>,
    pub level: usize,
}

impl Block {
    pub fn new(block_type: BlockType, parent_id: Option<BlockId>, level: usize) -> Self {
        Self {
            block_type,
            parent_id,
            children: Vec::new(),
            level,
        }
    }

    pub fn add_child(&mut self, child_id: BlockId) {
        self.children.push(child_id);
    }

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
