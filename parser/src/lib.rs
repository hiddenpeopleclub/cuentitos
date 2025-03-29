use cuentitos_common::*;

pub mod parser;
pub mod parsers;

pub use parser::*;

#[cfg(test)]
mod tests;

pub fn parse(script: &str) -> Result<Database, ParseError> {
    let mut parser = Parser::new();
    parser.parse(script)
}
