use cuentitos_common::Block;
use crate::ParseError;

pub struct Line<'a> {
  pub parsed: bool,
  pub text: &'a str,
}


#[derive(Debug)]
pub struct LineParserResult {
  pub block: Block,
  pub string: String,
}

pub fn parse(line: Line) -> Result<LineParserResult, ParseError> {
  Ok(
    LineParserResult {
      block: Block::String(1),
      string: String::from(line.text),
    }
  )
}

mod tests {
  use super::*;

#[test]
  fn test_parse() {
    let line = Line {
      parsed: false,
      text: "Hello, world!",
    };

    let result = super::parse(line).unwrap();
    assert_eq!(result.block, super::Block::String(1));
    assert_eq!(result.string, "Hello, world!");
  }
}
