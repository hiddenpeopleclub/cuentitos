use crate::{BlockId, StringId};

/// Metadata for a section in the narrative
#[derive(Debug, Clone, PartialEq)]
pub struct Section {
    pub block_id: BlockId, // The block ID of the Section block
    pub name: StringId,    // The section's display name
    pub path: StringId,    // The full hierarchical path (e.g., "Root \ Child")
}

impl Section {
    pub fn new(block_id: BlockId, name: StringId, path: StringId) -> Self {
        Self {
            block_id,
            name,
            path,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_section_creation() {
        let section = Section::new(1, 0, 1);
        assert_eq!(section.block_id, 1);
        assert_eq!(section.name, 0);
        assert_eq!(section.path, 1);
    }
}
