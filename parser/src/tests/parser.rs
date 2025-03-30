use crate::*;
use cuentitos_common::test_case::TestCase;

#[test]
fn test_single_line_script() {
    let test_case = TestCase::from_string(
        include_str!("../../../compatibility-tests/00000000001-single-line-and-end.md"),
        "single-line.md",
    );

    let mut parser = Parser::new();
    let database = parser.parse(&test_case.script).unwrap();

    assert_eq!(database.blocks.len(), 3);
    assert_eq!(database.strings.len(), 1);
}

#[test]
fn test_indented_script() {
    let test_case = TestCase::from_string(
        include_str!("../../../compatibility-tests/00000000009-nested-strings-with-siblings.md"),
        "nested-strings.md",
    );

    let mut parser = Parser::new();
    let database = parser.parse(&test_case.script).unwrap();

    assert_eq!(database.blocks.len(), 10); // START + 8 strings + END
    assert_eq!(database.strings.len(), 8);

    // Verify levels
    assert_eq!(database.blocks[1].level, 0); // Parent
    assert_eq!(database.blocks[2].level, 1); // First child
    assert_eq!(database.blocks[3].level, 2); // First grandchild
    assert_eq!(database.blocks[4].level, 2); // Second grandchild
    assert_eq!(database.blocks[5].level, 1); // Second child
    assert_eq!(database.blocks[6].level, 2); // Third grandchild
    assert_eq!(database.blocks[7].level, 2); // Fourth grandchild
    assert_eq!(database.blocks[8].level, 1); // Third child

    // Verify parent-child relationships
    assert_eq!(database.blocks[1].parent_id, Some(0)); // Parent -> START
    assert_eq!(database.blocks[2].parent_id, Some(1)); // First child -> Parent
    assert_eq!(database.blocks[3].parent_id, Some(2)); // First grandchild -> First child
    assert_eq!(database.blocks[4].parent_id, Some(2)); // Second grandchild -> First child
    assert_eq!(database.blocks[5].parent_id, Some(1)); // Second child -> Parent
    assert_eq!(database.blocks[6].parent_id, Some(5)); // Third grandchild -> Second child
    assert_eq!(database.blocks[7].parent_id, Some(5)); // Fourth grandchild -> Second child
    assert_eq!(database.blocks[8].parent_id, Some(1)); // Third child -> Parent
}

#[test]
fn test_invalid_indentation() {
    let mut parser = Parser::new();
    let result = parser.parse("   Hello");
    assert!(result.is_err());
    if let Err(ParseErrors(errors)) = result {
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            ParseError::InvalidIndentation { spaces, .. } => {
                assert_eq!(*spaces, 3);
            }
            _ => panic!("Expected InvalidIndentation error"),
        }
    } else {
        panic!("Expected ParseErrors");
    }
}

#[test]
fn test_skip_empty_lines() {
    let mut parser = Parser::new();
    let database = parser.parse("\n\nHello\n\n").unwrap();
    assert_eq!(database.blocks.len(), 3);
    assert_eq!(database.strings.len(), 1);
}

#[test]
fn test_invalid_indentation_with_file() {
    let mut parser = Parser::with_file("test.md");
    let result = parser.parse("   Hello");
    assert!(result.is_err());
    if let Err(ParseErrors(errors)) = result {
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            ParseError::InvalidIndentation { file, line, spaces } => {
                assert_eq!(file.to_str().unwrap(), "test.md");
                assert_eq!(*line, 1);
                assert_eq!(*spaces, 3);
            }
            _ => panic!("Expected InvalidIndentation error"),
        }
    } else {
        panic!("Expected ParseErrors");
    }
}

#[test]
fn test_nested_sections() {
    let mut parser = Parser::with_file("test.cuentitos");
    let script = "\
# Main Section
This is text in the main section
  ## First Sub-section
  This is text in the first sub-section
    ### Deep Sub-section
    This is text in a deep sub-section
  ## Second Sub-section
  This is text in the second sub-section

# Another Main Section
This is text in another main section
  ## Its Sub-section
  This is text in its sub-section";

    let database = parser.parse(script).unwrap();

    // Verify we have the expected number of blocks:
    // START + END + 2 root sections + 3 sub-sections + 1 deep sub-section + 6 text blocks = 14
    assert_eq!(database.blocks.len(), 14);

    // Find the block IDs for each section
    let find_section = |title: &str| -> BlockId {
        database.blocks.iter()
            .position(|block| matches!(block.block_type, BlockType::Section(id) if database.strings[id] == title))
            .unwrap()
    };

    let main_section = find_section("Main Section");
    let first_sub = find_section("First Sub-section");
    let deep_sub = find_section("Deep Sub-section");
    let second_sub = find_section("Second Sub-section");
    let another_main = find_section("Another Main Section");
    let its_sub = find_section("Its Sub-section");

    // Verify section levels
    assert_eq!(database.blocks[main_section].level, 0);
    assert_eq!(database.blocks[first_sub].level, 1);
    assert_eq!(database.blocks[deep_sub].level, 2);
    assert_eq!(database.blocks[second_sub].level, 1);
    assert_eq!(database.blocks[another_main].level, 0);
    assert_eq!(database.blocks[its_sub].level, 1);

    // Verify parent-child relationships
    assert_eq!(database.blocks[main_section].parent_id, Some(0)); // Main Section -> START
    assert_eq!(database.blocks[first_sub].parent_id, Some(main_section)); // First Sub-section -> Main Section
    assert_eq!(database.blocks[deep_sub].parent_id, Some(first_sub)); // Deep Sub-section -> First Sub-section
    assert_eq!(database.blocks[second_sub].parent_id, Some(main_section)); // Second Sub-section -> Main Section
    assert_eq!(database.blocks[another_main].parent_id, Some(0)); // Another Main Section -> START
    assert_eq!(database.blocks[its_sub].parent_id, Some(another_main)); // Its Sub-section -> Another Main Section

    // Verify text blocks are present with correct content
    let text_blocks: Vec<_> = database
        .blocks
        .iter()
        .filter_map(|block| {
            if let BlockType::String(id) = block.block_type {
                Some((block.level, database.strings[id].as_str()))
            } else {
                None
            }
        })
        .collect();

    assert_eq!(
        text_blocks,
        vec![
            (0, "This is text in the main section"),
            (1, "This is text in the first sub-section"),
            (2, "This is text in a deep sub-section"),
            (1, "This is text in the second sub-section"),
            (0, "This is text in another main section"),
            (1, "This is text in its sub-section"),
        ]
    );
}

#[test]
fn test_orphaned_subsection() {
    let mut parser = Parser::with_file("test.cuentitos");
    let script = "  ## Orphaned Sub-section\n  This should cause an error\n\n# Valid Section\nThis is text in a valid section\n  ## Valid Sub-section\n  This is valid";

    let result = parser.parse(script);
    assert!(result.is_err());

    if let Err(ParseErrors(errors)) = result {
        assert_eq!(errors.len(), 1);
        match &errors[0] {
            ParseError::OrphanedSubSection { file, line } => {
                assert_eq!(file.to_str().unwrap(), "test.cuentitos");
                assert_eq!(*line, 1);
            }
            _ => panic!("Expected OrphanedSubSection error"),
        }
    } else {
        panic!("Expected ParseErrors");
    }
}

#[test]
fn test_multiple_errors() {
    let mut parser = Parser::with_file("test.cuentitos");
    let script = "\
# Story
  ## Chapter One
  This is chapter one
  ## Chapter Two
  This is chapter two
  ## Chapter One
  This should cause an error
# Story
This should also cause an error";

    let result = parser.parse(script);
    assert!(result.is_err());

    // Convert the error into ParseErrors
    let errors = match result {
        Err(ParseErrors(errors)) => errors,
        _ => panic!("Expected ParseErrors"),
    };

    // Should have exactly 2 errors
    assert_eq!(errors.len(), 2);

    // First error should be duplicate Chapter One under Story
    match &errors[0] {
        ParseError::DuplicateSectionName {
            file,
            line,
            name,
            parent,
            previous_line,
        } => {
            assert_eq!(file.to_str().unwrap(), "test.cuentitos");
            assert_eq!(*line, 6);
            assert_eq!(name, "Chapter One");
            assert_eq!(parent, "Story");
            assert_eq!(*previous_line, 2);
        }
        _ => panic!("Expected DuplicateSectionName error"),
    }

    // Second error should be duplicate Story at root level
    match &errors[1] {
        ParseError::DuplicateSectionName {
            file,
            line,
            name,
            parent,
            previous_line,
        } => {
            assert_eq!(file.to_str().unwrap(), "test.cuentitos");
            assert_eq!(*line, 8);
            assert_eq!(name, "Story");
            assert_eq!(parent, "<root>");
            assert_eq!(*previous_line, 1);
        }
        _ => panic!("Expected DuplicateSectionName error"),
    }

    // Verify error messages match expected format
    let error_messages: Vec<_> = errors.iter().map(|e| e.to_string()).collect();
    assert_eq!(error_messages, vec![
        "test.cuentitos:6: ERROR: Duplicate section name: 'Chapter One' already exists at this level under 'Story'. Previously defined at line 2.",
        "test.cuentitos:8: ERROR: Duplicate section name: 'Story' already exists at this level under '<root>'. Previously defined at line 1."
    ]);
}
