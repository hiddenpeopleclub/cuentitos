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
