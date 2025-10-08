use crate::block::{Block, BlockId};
use crate::section::Section;
use crate::{SectionId, StringId};

#[derive(Debug, Default, Clone, PartialEq)]
pub struct Database {
    pub blocks: Vec<Block>,
    pub strings: Vec<String>,
    pub sections: Vec<Section>,
}

impl Database {
    pub fn new() -> Self {
        Self {
            blocks: Vec::new(),
            strings: Vec::new(),
            sections: Vec::new(),
        }
    }

    pub fn add_block(&mut self, block: Block) -> BlockId {
        let block_id = self.blocks.len();
        // If the block has a parent, add this block as a child to its parent
        if let Some(parent_id) = block.parent_id {
            if let Some(parent) = self.blocks.get_mut(parent_id) {
                parent.add_child(block_id);
            }
        }
        self.blocks.push(block);
        block_id
    }

    pub fn add_string(&mut self, string: String) -> StringId {
        let string_id = self.strings.len();
        self.strings.push(string);
        string_id
    }

    pub fn add_section(&mut self, section: Section) -> SectionId {
        let section_id = self.sections.len();
        self.sections.push(section);
        section_id
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::block::BlockType;

    #[test]
    fn test_database_add_block() {
        let mut db = Database::new();

        // Add root block
        let root = Block::new(BlockType::Start, None, 0);
        let root_id = db.add_block(root);
        assert_eq!(root_id, 0);

        // Add child block
        let child = Block::new(BlockType::String(0), Some(root_id), 1);
        let child_id = db.add_block(child);

        // Verify parent-child relationship
        assert_eq!(db.blocks[root_id].children, vec![child_id]);
        assert_eq!(db.blocks[child_id].parent_id, Some(root_id));
    }

    #[test]
    fn test_database_add_string() {
        let mut db = Database::new();
        let string_id = db.add_string("test".to_string());
        assert_eq!(string_id, 0);
        assert_eq!(db.strings[string_id], "test");
    }

    #[test]
    fn test_complex_hierarchy() {
        let mut db = Database::new();

        // Create a simple hierarchy:
        // root
        //   |- child1
        //   |   |- grandchild1
        //   |   |- grandchild2
        //   |- child2

        let root = Block::new(BlockType::Start, None, 0);
        let root_id = db.add_block(root);

        let child1 = Block::new(BlockType::String(0), Some(root_id), 1);
        let child1_id = db.add_block(child1);

        let child2 = Block::new(BlockType::String(1), Some(root_id), 1);
        let child2_id = db.add_block(child2);

        let grandchild1 = Block::new(BlockType::String(2), Some(child1_id), 2);
        let grandchild1_id = db.add_block(grandchild1);

        let grandchild2 = Block::new(BlockType::String(3), Some(child1_id), 2);
        let grandchild2_id = db.add_block(grandchild2);

        // Verify root's children
        assert_eq!(db.blocks[root_id].children, vec![child1_id, child2_id]);

        // Verify child1's children
        assert_eq!(
            db.blocks[child1_id].children,
            vec![grandchild1_id, grandchild2_id]
        );

        // Verify child2 has no children
        assert!(db.blocks[child2_id].is_leaf());

        // Verify levels
        assert_eq!(db.blocks[root_id].level, 0);
        assert_eq!(db.blocks[child1_id].level, 1);
        assert_eq!(db.blocks[child2_id].level, 1);
        assert_eq!(db.blocks[grandchild1_id].level, 2);
        assert_eq!(db.blocks[grandchild2_id].level, 2);
    }
}
