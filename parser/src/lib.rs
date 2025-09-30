use cuentitos_common::*;

pub mod block_parsers;
pub mod line_parser;
pub mod parser;

pub use parser::{ParseError, Parser};

pub fn parse(script: &str) -> Result<Database, ParseError> {
    let mut parser = Parser::new();
    parser.parse(script)
}
