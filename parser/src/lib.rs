use cuentitos_common::*;

pub mod boolean_expression;
pub mod expression;
pub mod parser;
pub mod parsers;

pub use parser::*;

pub fn parse(script: &str) -> Result<(Database, Vec<Warning>), ParseError> {
    let mut parser = Parser::new();
    parser.parse(script)
}
