use crate::{BlockId, BlockType, Database, SectionId};
use std::collections::HashMap;

/// Result of path resolution
#[derive(Debug, Clone, PartialEq)]
pub enum ResolvedPath {
    Section(SectionId),
    Start,
    Restart,
    End,
}

/// Errors that can occur during path resolution
#[derive(Debug, Clone, PartialEq)]
pub enum PathResolutionError {
    SectionNotFound { path: String },
    NavigationAboveRoot,
    InvalidPath { message: String },
}

/// Resolves section paths to SectionIds or special targets
pub struct PathResolver<'a> {
    database: &'a Database,
    section_registry: &'a HashMap<String, SectionId>,
}

impl<'a> PathResolver<'a> {
    pub fn new(database: &'a Database, section_registry: &'a HashMap<String, SectionId>) -> Self {
        Self {
            database,
            section_registry,
        }
    }

    /// Resolve a path string to a ResolvedPath
    ///
    /// Handles:
    /// - Special keywords: START, RESTART, END
    /// - Current section: .
    /// - Parent navigation: ..
    /// - Absolute paths: Root \ Child
    /// - Relative paths: Child, Sibling
    pub fn resolve_path(
        &self,
        path: &str,
        containing_section: Option<BlockId>,
    ) -> Result<ResolvedPath, PathResolutionError> {
        let path = path.trim();

        // Validate path doesn't end with backslash (trailing backslash)
        if path.ends_with(" \\") || path.ends_with('\\') {
            return Err(PathResolutionError::InvalidPath {
                message: "Expected section name after '->'".to_string(),
            });
        }

        // Validate path doesn't start with backslash (leading backslash)
        if path.starts_with("\\ ") || path.starts_with('\\') {
            return Err(PathResolutionError::InvalidPath {
                message: "Expected section name after '->'".to_string(),
            });
        }

        // Handle special keywords
        match path {
            "START" => return Ok(ResolvedPath::Start),
            "RESTART" => return Ok(ResolvedPath::Restart),
            "END" => return Ok(ResolvedPath::End),
            _ => {}
        }

        // Handle "." (current section)
        if path == "." {
            if let Some(section_block_id) = containing_section {
                // Get the SectionId from the block
                if let BlockType::Section(section_id) =
                    self.database.blocks[section_block_id].block_type
                {
                    return Ok(ResolvedPath::Section(section_id));
                }
            }
            return Err(PathResolutionError::SectionNotFound {
                path: path.to_string(),
            });
        }

        // Parse the path into segments
        let segments: Vec<&str> = path.split(" \\ ").map(|s| s.trim()).collect();

        // Validate that no segment is empty (handles trailing/leading backslashes)
        // Do this BEFORE checking absolute vs relative path
        for segment in &segments {
            if segment.is_empty() {
                return Err(PathResolutionError::InvalidPath {
                    message: "Expected section name after '->'".to_string(),
                });
            }
        }

        // Check if segments is empty (shouldn't happen with trim, but defensive)
        if segments.is_empty() {
            return Err(PathResolutionError::InvalidPath {
                message: "Expected section name after '->'".to_string(),
            });
        }

        // Check if this is an absolute path (doesn't start with ..)
        if !segments[0].starts_with("..") {
            // Try absolute path first
            if let Some(&section_id) = self.section_registry.get(path) {
                return Ok(ResolvedPath::Section(section_id));
            }

            // Try relative path (search children and siblings)
            if let Some(section_block_id) = containing_section {
                // Search children first
                if let Some(child_block_id) = self.find_child_section(section_block_id, segments[0])
                {
                    if segments.len() == 1 {
                        if let Some(section_id) = self.get_section_id(child_block_id) {
                            return Ok(ResolvedPath::Section(section_id));
                        }
                    }
                    // For longer paths, build the full path and look it up
                    let full_path = segments.join(" \\ ");
                    if let Some(&section_id) = self.section_registry.get(&full_path) {
                        return Ok(ResolvedPath::Section(section_id));
                    }
                }

                // Search siblings
                if let Some(sibling_block_id) =
                    self.find_sibling_section(section_block_id, segments[0])
                {
                    if segments.len() == 1 {
                        if let Some(section_id) = self.get_section_id(sibling_block_id) {
                            return Ok(ResolvedPath::Section(section_id));
                        }
                    }
                    // For longer paths, build the full path and look it up
                    let full_path = segments.join(" \\ ");
                    if let Some(&section_id) = self.section_registry.get(&full_path) {
                        return Ok(ResolvedPath::Section(section_id));
                    }
                }
            }
        } else {
            // Handle ".." navigation
            let mut current_section = containing_section;
            let mut segment_index = 0;

            // Process ".." segments
            while segment_index < segments.len() && segments[segment_index] == ".." {
                if let Some(section_block_id) = current_section {
                    // Navigate to parent section
                    current_section = self.find_parent_section(section_block_id);
                    if current_section.is_none() {
                        return Err(PathResolutionError::NavigationAboveRoot);
                    }
                } else {
                    return Err(PathResolutionError::NavigationAboveRoot);
                }
                segment_index += 1;
            }

            // If there are more segments, resolve them
            if segment_index < segments.len() {
                if let Some(section_block_id) = current_section {
                    // Look for the rest of the path as siblings
                    let remaining_path = segments[segment_index..].join(" \\ ");
                    if let Some(sibling_block_id) =
                        self.find_sibling_section(section_block_id, &remaining_path)
                    {
                        if let Some(section_id) = self.get_section_id(sibling_block_id) {
                            return Ok(ResolvedPath::Section(section_id));
                        }
                    }
                    // Try building the full path from current section
                    let current_path = self.build_section_path_string(section_block_id);
                    if let Some(parent_id) = self.database.blocks[section_block_id].parent_id {
                        let full_path = if current_path.is_empty() {
                            remaining_path.clone()
                        } else {
                            format!(
                                "{} \\ {}",
                                self.build_section_path_string(parent_id),
                                remaining_path
                            )
                        };
                        if let Some(&section_id) = self.section_registry.get(&full_path) {
                            return Ok(ResolvedPath::Section(section_id));
                        }
                    }
                }
            } else {
                // Just "..", return the parent section
                if let Some(section_block_id) = current_section {
                    if let Some(section_id) = self.get_section_id(section_block_id) {
                        return Ok(ResolvedPath::Section(section_id));
                    }
                }
            }
        }

        Err(PathResolutionError::SectionNotFound {
            path: path.to_string(),
        })
    }

    /// Find a direct child section of the given parent by display name
    fn find_child_section(&self, parent_id: BlockId, name: &str) -> Option<BlockId> {
        for &child_id in &self.database.blocks[parent_id].children {
            if let BlockType::Section(section_id) = &self.database.blocks[child_id].block_type {
                let section = &self.database.sections[*section_id];
                let section_name = &self.database.strings[section.name];
                if section_name == name {
                    return Some(child_id);
                }
            }
        }
        None
    }

    /// Find a sibling section (shares same parent) by display name
    fn find_sibling_section(&self, section_id: BlockId, name: &str) -> Option<BlockId> {
        if let Some(parent_id) = self.database.blocks[section_id].parent_id {
            for &sibling_id in &self.database.blocks[parent_id].children {
                if sibling_id != section_id {
                    if let BlockType::Section(sec_id) = &self.database.blocks[sibling_id].block_type
                    {
                        let section = &self.database.sections[*sec_id];
                        let section_name = &self.database.strings[section.name];
                        if section_name == name {
                            return Some(sibling_id);
                        }
                    }
                }
            }
        }
        None
    }

    /// Find the parent section of a given section
    fn find_parent_section(&self, section_id: BlockId) -> Option<BlockId> {
        let mut current_id = section_id;

        while let Some(parent_id) = self.database.blocks[current_id].parent_id {
            if matches!(
                self.database.blocks[parent_id].block_type,
                BlockType::Section(_)
            ) {
                return Some(parent_id);
            }
            current_id = parent_id;
        }

        None
    }

    /// Extract SectionId from a section BlockId
    fn get_section_id(&self, block_id: BlockId) -> Option<SectionId> {
        if let BlockType::Section(section_id) = self.database.blocks[block_id].block_type {
            Some(section_id)
        } else {
            None
        }
    }

    /// Build the full hierarchical path string for a section block
    fn build_section_path_string(&self, block_id: BlockId) -> String {
        let mut path_parts = Vec::new();
        let mut current_id = block_id;

        // Walk up the parent chain, collecting section names
        while let Some(parent_id) = self.database.blocks[current_id].parent_id {
            if let BlockType::Section(section_id) = &self.database.blocks[current_id].block_type {
                let section = &self.database.sections[*section_id];
                let name = &self.database.strings[section.name];
                path_parts.push(name.clone());
            }
            current_id = parent_id;
        }

        // Reverse to get top-down order
        path_parts.reverse();
        path_parts.join(" \\ ")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{Block, Section};

    fn create_test_database() -> Database {
        let mut db = Database::new();

        // Create: START -> Root -> Child A, Child B (Child B has Grandchild)
        let start = Block::new(BlockType::Start, None, 0);
        let start_id = db.add_block(start);

        // Root section
        let root_name_id = db.add_string("Root".to_string());
        let root_path_id = db.add_string("Root".to_string());
        let root_block = Block::new(BlockType::Section(0), Some(start_id), 0);
        let root_block_id = db.add_block(root_block);
        let root_section = Section::new(root_block_id, root_name_id, root_path_id);
        let root_section_id = db.add_section(root_section);
        db.blocks[root_block_id].block_type = BlockType::Section(root_section_id);
        db.section_registry
            .insert("Root".to_string(), root_section_id);

        // Child A
        let child_a_name_id = db.add_string("Child A".to_string());
        let child_a_path_id = db.add_string("Root \\ Child A".to_string());
        let child_a_block = Block::new(BlockType::Section(0), Some(root_block_id), 1);
        let child_a_block_id = db.add_block(child_a_block);
        let child_a_section = Section::new(child_a_block_id, child_a_name_id, child_a_path_id);
        let child_a_section_id = db.add_section(child_a_section);
        db.blocks[child_a_block_id].block_type = BlockType::Section(child_a_section_id);
        db.section_registry
            .insert("Root \\ Child A".to_string(), child_a_section_id);

        // Child B
        let child_b_name_id = db.add_string("Child B".to_string());
        let child_b_path_id = db.add_string("Root \\ Child B".to_string());
        let child_b_block = Block::new(BlockType::Section(0), Some(root_block_id), 1);
        let child_b_block_id = db.add_block(child_b_block);
        let child_b_section = Section::new(child_b_block_id, child_b_name_id, child_b_path_id);
        let child_b_section_id = db.add_section(child_b_section);
        db.blocks[child_b_block_id].block_type = BlockType::Section(child_b_section_id);
        db.section_registry
            .insert("Root \\ Child B".to_string(), child_b_section_id);

        // Grandchild under Child B
        let grandchild_name_id = db.add_string("Grandchild".to_string());
        let grandchild_path_id = db.add_string("Root \\ Child B \\ Grandchild".to_string());
        let grandchild_block = Block::new(BlockType::Section(0), Some(child_b_block_id), 2);
        let grandchild_block_id = db.add_block(grandchild_block);
        let grandchild_section =
            Section::new(grandchild_block_id, grandchild_name_id, grandchild_path_id);
        let grandchild_section_id = db.add_section(grandchild_section);
        db.blocks[grandchild_block_id].block_type = BlockType::Section(grandchild_section_id);
        db.section_registry.insert(
            "Root \\ Child B \\ Grandchild".to_string(),
            grandchild_section_id,
        );

        db
    }

    #[test]
    fn test_resolve_special_keywords() {
        let db = create_test_database();
        let resolver = PathResolver::new(&db, &db.section_registry);

        assert_eq!(
            resolver.resolve_path("START", None),
            Ok(ResolvedPath::Start)
        );
        assert_eq!(
            resolver.resolve_path("RESTART", None),
            Ok(ResolvedPath::Restart)
        );
        assert_eq!(resolver.resolve_path("END", None), Ok(ResolvedPath::End));
    }

    #[test]
    fn test_resolve_absolute_path() {
        let db = create_test_database();
        let resolver = PathResolver::new(&db, &db.section_registry);

        // Resolve "Root"
        if let Ok(ResolvedPath::Section(section_id)) = resolver.resolve_path("Root", None) {
            assert_eq!(section_id, 0); // First section added
        } else {
            panic!("Expected Section(0)");
        }

        // Resolve "Root \ Child A"
        if let Ok(ResolvedPath::Section(section_id)) =
            resolver.resolve_path("Root \\ Child A", None)
        {
            assert_eq!(section_id, 1);
        } else {
            panic!("Expected Section(1)");
        }
    }

    #[test]
    fn test_resolve_relative_child() {
        let db = create_test_database();
        let resolver = PathResolver::new(&db, &db.section_registry);

        // From Root (block_id=1), resolve "Child A"
        let root_block_id = 1;
        if let Ok(ResolvedPath::Section(section_id)) =
            resolver.resolve_path("Child A", Some(root_block_id))
        {
            assert_eq!(section_id, 1); // Child A section
        } else {
            panic!("Expected Section(1)");
        }
    }

    #[test]
    fn test_resolve_current_section() {
        let db = create_test_database();
        let resolver = PathResolver::new(&db, &db.section_registry);

        // From Root (block_id=1), resolve "."
        let root_block_id = 1;
        if let Ok(ResolvedPath::Section(section_id)) =
            resolver.resolve_path(".", Some(root_block_id))
        {
            assert_eq!(section_id, 0); // Root section
        } else {
            panic!("Expected Section(0)");
        }
    }

    #[test]
    fn test_resolve_parent_navigation() {
        let db = create_test_database();
        let resolver = PathResolver::new(&db, &db.section_registry);

        // From Child A (block_id=2), resolve ".."
        let child_a_block_id = 2;
        if let Ok(ResolvedPath::Section(section_id)) =
            resolver.resolve_path("..", Some(child_a_block_id))
        {
            assert_eq!(section_id, 0); // Root section
        } else {
            panic!("Expected Section(0)");
        }
    }

    #[test]
    fn test_error_section_not_found() {
        let db = create_test_database();
        let resolver = PathResolver::new(&db, &db.section_registry);

        assert_eq!(
            resolver.resolve_path("Nonexistent", None),
            Err(PathResolutionError::SectionNotFound {
                path: "Nonexistent".to_string()
            })
        );
    }

    #[test]
    fn test_error_navigation_above_root() {
        let db = create_test_database();
        let resolver = PathResolver::new(&db, &db.section_registry);

        // From Root (block_id=1), try to go up with ".."
        let root_block_id = 1;
        assert_eq!(
            resolver.resolve_path("..", Some(root_block_id)),
            Err(PathResolutionError::NavigationAboveRoot)
        );
    }

    #[test]
    fn test_error_invalid_path_trailing_backslash() {
        let db = create_test_database();
        let resolver = PathResolver::new(&db, &db.section_registry);

        assert_eq!(
            resolver.resolve_path("Root \\", None),
            Err(PathResolutionError::InvalidPath {
                message: "Expected section name after '->'".to_string()
            })
        );
    }
}
