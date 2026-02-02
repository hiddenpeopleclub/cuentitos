use crate::{ComparisonOperator, SectionId, StringId, VariableId, VariableValue};

pub type BlockId = usize;

#[derive(Debug, Clone, PartialEq)]
pub enum BlockType {
    Start,
    String(StringId),
    Section(SectionId),
    Option(StringId),
    GoTo(SectionId),
    GoToAndBack(SectionId),
    GoToStart,
    GoToRestart,
    GoToEnd,
    SetVariable {
        variable_id: VariableId,
        value: VariableValue,
    },
    RequireVariable {
        variable_id: VariableId,
        operator: ComparisonOperator,
        value: VariableValue,
    },
    End,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Block {
    pub block_type: BlockType,
    pub parent_id: Option<BlockId>,
    pub children: Vec<BlockId>,
    pub level: usize,
    pub line: usize, // Line number in the source file (0 for generated blocks)
}

impl Block {
    pub fn new(block_type: BlockType, parent_id: Option<BlockId>, level: usize) -> Self {
        Self {
            block_type,
            parent_id,
            children: Vec::new(),
            level,
            line: 0,
        }
    }

    pub fn with_line(
        block_type: BlockType,
        parent_id: Option<BlockId>,
        level: usize,
        line: usize,
    ) -> Self {
        Self {
            block_type,
            parent_id,
            children: Vec::new(),
            level,
            line,
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

    #[test]
    fn test_option_block_type() {
        let block = Block::new(BlockType::Option(0), Some(1), 1);
        assert_eq!(block.parent_id, Some(1));
        assert_eq!(block.level, 1);
        match block.block_type {
            BlockType::Option(string_id) => assert_eq!(string_id, 0),
            _ => panic!("Expected Option block type"),
        }
    }
}
