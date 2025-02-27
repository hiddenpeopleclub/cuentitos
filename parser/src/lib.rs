use cuentitos_common::*;

mod parser;
use parser::*;

mod line_parser;

pub fn parse(script: &str) -> Result<Database, ParseError> {
    let mut parser = Parser;
    parser.parse(script)
}
