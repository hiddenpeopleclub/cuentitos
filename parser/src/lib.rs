use cuentitos_common::*;

mod parser;
pub use parser::{Parser, ParseError};

mod line_parser;

pub fn parse(script: &str) -> Result<Database, ParseError> {
    let mut parser = Parser::new();
    parser.parse(script)
}
