#[cfg(test)]
mod test {
    use crate::parse;
    use cuentitos_common::{test_case::TestCase, BlockType};

    #[test]
    fn block_creation() {
        let test_case = TestCase::from_string(
            include_str!("../../../compatibility-tests/00000000011-basic-section.md"),
            "basic-section.md",
        );

        let database = parse(&test_case.script).unwrap();
        let block = &database.blocks[1]; // After START block

        assert!(
            matches!(block.block_type, BlockType::Section(id) if database.strings[id] == "First Section")
        );
        assert_eq!(block.level, 0);
        assert_eq!(block.parent_id, Some(0)); // Parent is START
    }

    #[test]
    fn title_in_strings_table() {
        let test_case = TestCase::from_string(
            include_str!("../../../compatibility-tests/00000000012-nested-sections.md"),
            "nested-sections.md",
        );

        let database = parse(&test_case.script).unwrap();

        // Find section blocks and verify their titles are in strings table
        let sections: Vec<_> = database
            .blocks
            .iter()
            .filter_map(|b| match b.block_type {
                BlockType::Section(id) => Some(database.strings[id].clone()),
                _ => None,
            })
            .collect();

        assert_eq!(
            sections,
            vec![
                "Main Section",
                "First Sub-section",
                "Deep Sub-section",
                "Second Sub-section",
                "Another Main Section",
                "Its Sub-section"
            ]
        );
    }

    #[test]
    fn level_calculation() {
        let test_case = TestCase::from_string(
            include_str!("../../../compatibility-tests/00000000012-nested-sections.md"),
            "nested-sections.md",
        );

        let database = parse(&test_case.script).unwrap();

        // Get all section blocks
        let sections: Vec<_> = database
            .blocks
            .iter()
            .filter(|b| matches!(b.block_type, BlockType::Section(_)))
            .collect();

        assert_eq!(sections[0].level, 0); // Main Section
        assert_eq!(sections[1].level, 1); // First Sub-section
        assert_eq!(sections[2].level, 2); // Deep Sub-section
        assert_eq!(sections[3].level, 1); // Second Sub-section
        assert_eq!(sections[4].level, 0); // Another Main Section
        assert_eq!(sections[5].level, 1); // Its Sub-section
    }

    #[test]
    fn indentation_validation() {
        let test_case = TestCase::from_string(
            include_str!("../../../compatibility-tests/00000000015-invalid-section-indentation.md"),
            "invalid-section-indentation.md",
        );

        let result = parse(&test_case.script);
        assert!(result.is_err());

        let test_case = TestCase::from_string(
            include_str!("../../../compatibility-tests/00000000012-nested-sections.md"),
            "nested-sections.md",
        );

        let result = parse(&test_case.script);
        assert!(result.is_ok());
    }

    #[test]
    fn block_relationships() {
        let test_case = TestCase::from_string(
            include_str!("../../../compatibility-tests/00000000012-nested-sections.md"),
            "nested-sections.md",
        );

        let database = parse(&test_case.script).unwrap();

        // Find the blocks
        let main_section = database.blocks.iter()
            .position(|b| matches!(b.block_type, BlockType::Section(id) if database.strings[id] == "Main Section"))
            .unwrap();

        let first_sub = database.blocks.iter()
            .position(|b| matches!(b.block_type, BlockType::Section(id) if database.strings[id] == "First Sub-section"))
            .unwrap();

        let deep_sub = database.blocks.iter()
            .position(|b| matches!(b.block_type, BlockType::Section(id) if database.strings[id] == "Deep Sub-section"))
            .unwrap();

        let second_sub = database.blocks.iter()
            .position(|b| matches!(b.block_type, BlockType::Section(id) if database.strings[id] == "Second Sub-section"))
            .unwrap();

        let another_main = database.blocks.iter()
            .position(|b| matches!(b.block_type, BlockType::Section(id) if database.strings[id] == "Another Main Section"))
            .unwrap();

        let its_sub = database.blocks.iter()
            .position(|b| matches!(b.block_type, BlockType::Section(id) if database.strings[id] == "Its Sub-section"))
            .unwrap();

        // Verify parent relationships
        assert_eq!(
            database.blocks[first_sub].parent_id,
            Some(main_section),
            "First Sub-section should be child of Main Section"
        );
        assert_eq!(
            database.blocks[deep_sub].parent_id,
            Some(first_sub),
            "Deep Sub-section should be child of First Sub-section"
        );
        assert_eq!(
            database.blocks[second_sub].parent_id,
            Some(main_section),
            "Second Sub-section should be child of Main Section"
        );
        assert_eq!(
            database.blocks[another_main].parent_id,
            Some(0),
            "Another Main Section should be child of START"
        );
        assert_eq!(
            database.blocks[its_sub].parent_id,
            Some(another_main),
            "Its Sub-section should be child of Another Main Section"
        );

        // Verify children relationships
        assert!(
            database.blocks[main_section].children.contains(&first_sub),
            "Main Section should have First Sub-section as child"
        );
        assert!(
            database.blocks[main_section].children.contains(&second_sub),
            "Main Section should have Second Sub-section as child"
        );
        assert!(
            database.blocks[first_sub].children.contains(&deep_sub),
            "First Sub-section should have Deep Sub-section as child"
        );
        assert!(
            database.blocks[another_main].children.contains(&its_sub),
            "Another Main Section should have Its Sub-section as child"
        );

        // Verify levels are correct
        assert_eq!(database.blocks[main_section].level, 0);
        assert_eq!(database.blocks[first_sub].level, 1);
        assert_eq!(database.blocks[deep_sub].level, 2);
        assert_eq!(database.blocks[second_sub].level, 1);
        assert_eq!(database.blocks[another_main].level, 0);
        assert_eq!(database.blocks[its_sub].level, 1);
    }

    #[test]
    fn sibling_sections_have_same_parent() {
        let script = "\
# Main Section
  ## First Sub-section
  This is text in the first sub-section
  ## Second Sub-section
  This is text in the second sub-section";

        let database = parse(script).unwrap();

        // Find the blocks
        let main_section = database.blocks.iter()
            .position(|b| matches!(b.block_type, BlockType::Section(id) if database.strings[id] == "Main Section"))
            .unwrap();

        let first_sub = database.blocks.iter()
            .position(|b| matches!(b.block_type, BlockType::Section(id) if database.strings[id] == "First Sub-section"))
            .unwrap();

        let second_sub = database.blocks.iter()
            .position(|b| matches!(b.block_type, BlockType::Section(id) if database.strings[id] == "Second Sub-section"))
            .unwrap();

        // Both sub-sections should have Main Section as their parent
        assert_eq!(database.blocks[first_sub].parent_id, Some(main_section));
        assert_eq!(database.blocks[second_sub].parent_id, Some(main_section));

        // Main Section should have both sub-sections as children
        assert!(database.blocks[main_section].children.contains(&first_sub));
        assert!(database.blocks[main_section].children.contains(&second_sub));

        // Both sub-sections should be at the same level
        assert_eq!(
            database.blocks[first_sub].level,
            database.blocks[second_sub].level
        );
    }
}
