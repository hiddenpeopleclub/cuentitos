/// Block parsers are modules that parse a line of text into blocks and strings.
/// Each block parser should be implemented as a separate module with a struct that
/// provides a static `parse` method with the following signature:
///
/// ```rust,ignore
/// pub struct MyParser;
///
/// impl MyParser {
///     pub fn parse(line: line_parser::Line, level: usize) -> Option<(Vec<Block>, Vec<String>)> {
///         // Parse the line and return:
///         // - None if this parser cannot handle this line
///         // - Some((blocks, strings)) if this parser can handle the line, where:
///         //   - blocks: The blocks created from this line
///         //   - strings: The strings referenced by the blocks
///         //
///         // Note: String blocks should use a temporary ID of 0, which will be
///         // updated by the main parser when adding the strings to the database.
///     }
/// }
/// ```
///
/// The parser should:
/// 1. Return None if it cannot parse the line
/// 2. Return Some with the blocks and strings if it can parse the line
/// 3. Set any String block IDs to 0 (they will be updated by the main parser)
/// 4. Use the provided level for block indentation
/// 5. Handle empty lines by returning None
mod section;
mod string;

pub use section::SectionParser;
pub use string::StringParser;
