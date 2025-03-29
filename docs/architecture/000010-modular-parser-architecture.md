# Modular Parser Architecture

### Submitters

- Fran Tufro

## Change Log

- [pending] 2024-03-25

## Context

The cuentitos game narrative engine needs a modular and extensible parsing system to handle different language features independently. The current monolithic parser design makes it difficult to add new features and maintain the codebase. We need a way to separate concerns and allow for independent development of different parsing features.

## Proposed Design

### Core Changes

1. Parser Organization
   - Create a new `src/parsers` directory in the parser crate
   - Move existing parsers into separate modules
   - Each parser will be responsible for a specific language feature
   - Implement a clear interface for all parsers to follow

2. Parser Types and Responsibilities
   - Line Parser: Handles basic text line parsing (existing)
   - Section Parser: Will handle section declarations and boundaries (future)
   - Choice Parser: Will handle choice blocks and their options (future)
   - More parsers can be added as new features are needed

3. Parser Interface
   ```rust
   trait FeatureParser {
       type Output;
       type Error;

       fn parse(&self, input: &str, context: &ParserContext) -> Result<Self::Output, Self::Error>;
   }
   ```

4. Parser Execution Flow
   - Main parser will orchestrate the execution of feature parsers
   - Parsers will be called in a specific, manually defined order
   - Each parser can access the results of previous parsers through the context
   - Results are accumulated and combined into the final AST

5. Example Parser Pipeline
   ```rust
   // Pseudo-code for the main parse method
   fn parse(&mut self, script: &str) -> Result<Database, ParseError> {
       let mut context = ParserContext::new();

       // Execute parsers in specific order
       let line_results = self.line_parser.parse(script, &context)?;
       context.add_results(line_results);

       let section_results = self.section_parser.parse(script, &context)?;
       context.add_results(section_results);

       // More parsers as needed...

       self.build_final_ast(context)
   }
   ```

### Implementation Details

1. Core Trait and Context
   ```rust
   pub trait FeatureParser {
       type Output;
       type Error;

       fn parse(&self, input: &str, context: &mut ParserContext) -> Result<Self::Output, Self::Error>;
   }

   pub struct ParserContext {
       current_line: usize,
       current_level: usize,
       file_path: Option<PathBuf>,
       database: Database,
   }
   ```

2. Parser Organization
   - Created `src/parsers/` directory to house all parser implementations
   - Moved line parser to `src/parsers/line_parser.rs`
   - Added `mod.rs` to define shared traits and types
   - Each parser module is declared in `parsers/mod.rs`

3. State Management
   - Moved parsing state from individual parsers to shared `ParserContext`
   - Line numbers start at 1 and increment as parsing progresses
   - Indentation level tracking for proper block nesting
   - File path information for error reporting
   - Shared database instance for building the AST

4. Main Parser Changes
   ```rust
   pub struct Parser {
       last_block_at_level: Vec<BlockId>,
       file_path: Option<PathBuf>,
       line_parser: LineParser,
       // Future parsers will be added here
   }
   ```
   - Main parser owns individual feature parsers
   - Orchestrates the parsing pipeline
   - Manages block hierarchy through `last_block_at_level`
   - Delegates line-specific parsing to `LineParser`

5. Error Handling
   - Each parser can define its own error type
   - Errors include file and line information for better debugging
   - Common error types (like `InvalidIndentation`) are shared

6. Testing Strategy
   - Unit tests for individual parsers
   - Integration tests for the complete parsing pipeline
   - Test cases include:
     - Basic line parsing
     - Indentation validation
     - Empty line handling
     - File path reporting in errors

### Benefits

1. Modularity
   - Each parser is focused on a single responsibility
   - Easier to test individual parsing features
   - Simpler to maintain and modify existing parsers

2. Extensibility
   - New language features can be added by creating new parsers
   - Existing parsers can be modified without affecting others
   - Clear separation of concerns

3. Testability
   - Each parser can be tested in isolation
   - Easier to write unit tests for specific features
   - Better error isolation and debugging

### Migration Plan

1. Phase 1: Initial Structure
   - Create `src/parsers` directory
   - Move existing line parser to `src/parsers/line_parser.rs`
   - Create base parser trait and interfaces

2. Phase 2: Parser Context
   - Implement parser context to share state between parsers
   - Add support for accumulating results

3. Phase 3: New Parsers
   - Implement new parsers as needed
   - Add them to the main parser pipeline
   - Write comprehensive tests for each

## Other Related ADRs

- [Lines of Text](000005-lines-of-text.md) - Text handling that will be affected by this change
- [Indentation and Block Parenting](000009-indentation-block-parenting.md) - Block structure that parsers need to respect

## References

- [Rust Parser Combinators](https://bodil.lol/parser-combinators/) - Inspiration for parser design
- [Compiler Design Patterns](https://www.aosabook.org/en/500L/a-python-interpreter-written-in-python.html) - General parsing architecture patterns
