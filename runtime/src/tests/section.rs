#[cfg(test)]
mod test {
    use crate::Runtime;
    use cuentitos_common::{BlockType, test_case::TestCase};

    #[test]
    fn test_basic_section_traversal() {
        let test_case = TestCase::from_string(
            include_str!("../../../compatibility-tests/00000000011-basic-section.md"),
            "00000000011-basic-section.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();
        let mut runtime = Runtime::new(database);

        runtime.run();

        // Should start at START block
        assert!(matches!(runtime.current_block().unwrap().block_type, BlockType::Start));

        // Step to First Section
        runtime.step();
        if let Some(block) = runtime.current_block() {
            match block.block_type {
                BlockType::Section(id) => assert_eq!(runtime.database.strings[id], "First Section"),
                _ => panic!("Expected Section block"),
            }
        }

        // Step through content of First Section
        runtime.step();
        if let Some(block) = runtime.current_block() {
            match block.block_type {
                BlockType::String(id) => assert_eq!(runtime.database.strings[id], "This is text in the first section"),
                _ => panic!("Expected String block"),
            }
        }

        runtime.step();
        if let Some(block) = runtime.current_block() {
            match block.block_type {
                BlockType::String(id) => assert_eq!(runtime.database.strings[id], "This is more text in the first section"),
                _ => panic!("Expected String block"),
            }
        }

        // Step to Second Section
        runtime.step();
        if let Some(block) = runtime.current_block() {
            match block.block_type {
                BlockType::Section(id) => assert_eq!(runtime.database.strings[id], "Second Section"),
                _ => panic!("Expected Section block"),
            }
        }

        // Step through content of Second Section
        runtime.step();
        if let Some(block) = runtime.current_block() {
            match block.block_type {
                BlockType::String(id) => assert_eq!(runtime.database.strings[id], "This is text in the second section"),
                _ => panic!("Expected String block"),
            }
        }

        // Final step should reach End block
        runtime.step();
        assert!(matches!(runtime.current_block().unwrap().block_type, BlockType::End));
    }

    #[test]
    fn test_nested_section_traversal() {
        let test_case = TestCase::from_string(
            include_str!("../../../compatibility-tests/00000000012-nested-sections.md"),
            "nested-sections.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();
        let mut runtime = Runtime::new(database);

        runtime.run();

        // START block
        assert!(matches!(runtime.current_block().unwrap().block_type, BlockType::Start));

        // Main Section
        runtime.step();
        let sections = runtime.current_section_hierarchy();
        assert_eq!(sections.len(), 1);
        assert!(matches!(&sections[0].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "Main Section"));

        // Skip "This is text in the main section"
        runtime.step();

        // First Sub-section
        runtime.step();
        let sections = runtime.current_section_hierarchy();
        assert_eq!(sections.len(), 2);
        assert!(matches!(&sections[0].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "Main Section"));
        assert!(matches!(&sections[1].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "First Sub-section"));

        // Skip "This is text in the first sub-section"
        runtime.step();

        // Deep Sub-section
        runtime.step();
        let sections = runtime.current_section_hierarchy();
        assert_eq!(sections.len(), 3);
        assert!(matches!(&sections[0].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "Main Section"));
        assert!(matches!(&sections[1].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "First Sub-section"));
        assert!(matches!(&sections[2].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "Deep Sub-section"));

        // Skip "This is text in a deep sub-section"
        runtime.step();

        // Second Sub-section
        runtime.step();
        let sections = runtime.current_section_hierarchy();
        assert_eq!(sections.len(), 2);
        assert!(matches!(&sections[0].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "Main Section"));
        assert!(matches!(&sections[1].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "Second Sub-section"));

        // Skip "This is text in the second sub-section"
        runtime.step();

        // Another Main Section
        runtime.step();
        let sections = runtime.current_section_hierarchy();
        assert_eq!(sections.len(), 1);
        assert!(matches!(&sections[0].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "Another Main Section"));

        // Skip "This is text in another main section"
        runtime.step();

        // Its Sub-section
        runtime.step();
        let sections = runtime.current_section_hierarchy();
        assert_eq!(sections.len(), 2);
        assert!(matches!(&sections[0].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "Another Main Section"));
        assert!(matches!(&sections[1].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "Its Sub-section"));

        // Skip "This is text in its sub-section"
        runtime.step();

        // END block
        runtime.step();
        assert!(matches!(runtime.current_block().unwrap().block_type, BlockType::End));
    }

    #[test]
    fn test_current_blocks_with_sections() {
        let test_case = TestCase::from_string(
            include_str!("../../../compatibility-tests/00000000011-basic-section.md"),
            "00000000011-basic-section.md",
        );

        let database = cuentitos_parser::parse(&test_case.script).unwrap();
        let mut runtime = Runtime::new(database);

        runtime.run();

        // Initially should only see START
        assert_eq!(runtime.current_blocks().len(), 1);
        assert!(matches!(runtime.current_blocks()[0].block_type, BlockType::Start));

        // Step to first section
        runtime.step();
        let blocks = runtime.current_blocks();
        assert_eq!(blocks.len(), 2);
        assert!(matches!(blocks[0].block_type, BlockType::Start));
        assert!(matches!(&blocks[1].block_type, BlockType::Section(id) if runtime.database.strings[*id] == "First Section"));

        // Skip to Second Section
        while !matches!(runtime.current_block().unwrap().block_type,
                       BlockType::Section(id) if runtime.database.strings[id] == "Second Section") {
            runtime.step();
        }

        // Should see all blocks up to Second Section
        let blocks = runtime.current_blocks();
        let texts: Vec<_> = blocks.iter()
            .filter_map(|b| match b.block_type {
                BlockType::String(id) | BlockType::Section(id) => Some(runtime.database.strings[id].clone()),
                _ => None
            })
            .collect();

        assert_eq!(texts, vec![
            "First Section",
            "This is text in the first section",
            "This is more text in the first section",
            "Second Section"
        ]);
    }
}
