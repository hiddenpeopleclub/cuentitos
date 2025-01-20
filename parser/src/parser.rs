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

    Ok(database)
  }
}
