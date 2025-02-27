use crate::line_parser;
use cuentitos_common::*;

#[derive(Debug, Default)]
pub struct Parser;

#[derive(Debug)]
#[derive(Clone)]
pub enum ParseError {
  UnexpectedToken,
  UnexpectedEndOfFile,
}

impl Parser {
  pub fn parse<A>(&mut self, script: A) -> Result<Database, ParseError>
  where A: AsRef<str>
  {
    let mut database = Database::default();

    let script = script.as_ref();

    database.blocks.push(Block::Start);

    // iterate through each line
    for line in script.lines() {
      let line = line_parser::Line { parsed: false, text: line };
      let result = line_parser::parse(line);

      match result {
        Ok(result) => {
          database.blocks.push(Block::String(database.strings.len()));
          database.strings.push(result.string);
        },
        Err(_) => panic!("Error parsing line"),
      }
    }

    database.blocks.push(Block::End);

    Ok(database)
  }
}

#[cfg(test)]
mod test {
  use cuentitos_common::test_case::TestCase;
  use super::*;

  #[test]
  fn test_single_line_script() {
    let test_case = TestCase::from_string(
      include_str!("../../compatibility-tests/00000000001-single-line-and-end.md"),
      "single-line.md"
    );

    let mut parser = Parser::default();
    let database = parser.parse(test_case.script).unwrap();
    assert_eq!(database.blocks.len(), 3);
    assert_eq!(database.strings.len(), 1);
    assert_eq!(database.blocks[0], Block::Start);
    assert_eq!(database.blocks[1], Block::String(0));
    assert_eq!(database.strings[0], "This is a single line");
    assert_eq!(database.blocks[2], Block::End);

  }
}
