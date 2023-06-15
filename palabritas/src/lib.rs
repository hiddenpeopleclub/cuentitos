mod parser;
pub use parser::parse_file;
pub use parser::parse_file_from_path;
#[macro_use]
extern crate pest_derive;

mod error;

use std::path::Path;

use cuentitos_common::{
  Block, BlockSettings, Condition, Database, FrequencyModifier, Modifier, NextBlock, Operator,
  Requirement,
};
use pest::{iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "palabritas.pest"]
pub struct PalabritasParser;

pub fn parse_database_from_path<P>(path: P) -> Option<Database>
where
  P: AsRef<Path>,
{
  let str = std::fs::read_to_string(&path).expect(format!("cannot read file: {}", path.as_ref().display()).as_str());
  let token = PalabritasParser::parse(Rule::File, &str)
    .expect("unsuccessful parse") // unwrap the parse result
    .next()
    .unwrap();

  parse_file(token)
}

pub fn parse_file(token: Pair<Rule>) -> Option<Database> {
  if token.as_rule() != Rule::File {
    return None;
  }

  let mut blocks: Vec<Vec<Block>> = Vec::default();
  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Block {
      parse_block(inner_token, &mut blocks, 0);
    }
  }

  if blocks.is_empty() {
    return None;
  }

  if let Some(last) = blocks[0].last_mut() {
    last.get_settings_mut().next = NextBlock::EndOfFile;
  }

  let mut ordered_blocks = Vec::default();

  for child_level in 0..blocks.len() {
    let mut index_offset = 0;
    for childs_in_level in blocks.iter().take(child_level + 1) {
      index_offset += childs_in_level.len();
    }

    for block in &mut blocks[child_level] {
      let settings = block.get_settings_mut();
      for child in &mut settings.children {
        *child += index_offset;
      }

      ordered_blocks.push(block.clone());
    }
  }

  Some(Database {
    blocks: ordered_blocks,
  })
}

fn parse_block(token: Pair<Rule>, blocks: &mut Vec<Vec<Block>>, child_order: usize) {
  if token.as_rule() != Rule::Block {
    return;
  }

  //    (NamedBucket | Choice | Text)  ~  " "* ~ Command* ~ " "* ~ (NEWLINE | EOI) ~ NewBlock*
  let mut block = Block::default();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Text => {
        if let Some(text) = parse_text(inner_token) {
          block = text;
        }
      }
      /*Rule::NamedBucket => {
        if let Some(named_bucket) = parse_named_bucket(inner_token) {
          block = named_bucket;
        }
      } */
      Rule::Choice => {
        if let Some(choice) = parse_choice(inner_token) {
          block = choice;
        }
      }
      Rule::Command => {
        add_command_to_block(inner_token, &mut block);
      }
      Rule::NewBlock => {
        for inner_blocks_token in get_blocks_from_new_block(inner_token) {
          let settings = block.get_settings_mut();
          parse_block(inner_blocks_token, blocks, child_order + 1);
          settings.children.push(blocks[child_order + 1].len() - 1);
        }
      }
      _ => {}
    }
  }

  while child_order >= blocks.len() {
    blocks.push(Vec::default());
  }

  blocks[child_order].push(block);
}

fn get_blocks_from_new_block(token: Pair<Rule>) -> Vec<Pair<Rule>> {
  let mut blocks = Vec::default();

  if token.as_rule() != Rule::NewBlock {
    return blocks;
  }

  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Block {
      blocks.push(inner_token);
    }
  }
  blocks
}
/*
fn parse_named_bucket(token: Pair<Rule>) -> Option<Block> {
  if token.as_rule() != Rule::NamedBucket {
    return None;
  }

  let mut blocks = Block {
   // blocks_type: BlockType::NamedBucket,
    ..Default::default()
  };

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Probability => {
        blocks.probability = parse_probability(inner_token);
      }
      Rule::SnakeCase => {
        blocks.text = inner_token.as_str().to_string();
      }
      _ => {}
    }
  }

  Some(blocks)
}
*/
fn parse_choice(token: Pair<Rule>) -> Option<Block> {
  if token.as_rule() != Rule::Choice {
    return None;
  }

  let mut text = String::default();
  let mut settings = BlockSettings::default();

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Probability => {
        settings.chance = parse_probability(inner_token);
      }
      Rule::String => {
        text = inner_token.as_str().to_string();
      }
      _ => {}
    }
  }

  Some(Block::Choice { id: text, settings })
}

fn parse_text(token: Pair<Rule>) -> Option<Block> {
  if token.as_rule() != Rule::Text {
    return None;
  }

  let mut text = String::default();
  let mut settings = BlockSettings::default();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Probability => {
        settings.chance = parse_probability(inner_token);
      }
      Rule::String => {
        text = inner_token.as_str().to_string();
      }
      _ => {}
    }
  }

  Some(Block::Text { id: text, settings })
}

fn add_command_to_settings(token: Pair<Rule>, settings: &mut BlockSettings) {
  if token.as_rule() != Rule::Command {
    return;
  }

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      //Command = {NEWLINE ~ (Indentation | " ")* ~ (Requirement | Frequency | Modifier | Divert) }
      Rule::Requirement => {
        if let Some(requirement) = parse_requirement(inner_token) {
          settings.requirements.push(requirement);
        }
      }
      Rule::Frequency => {
        if let Some(frequency) = parse_frequency(inner_token) {
          settings.frequency_modifiers.push(frequency);
        }
      }
      Rule::Modifier => {
        if let Some(modifier) = parse_modifier(inner_token) {
          settings.modifiers.push(modifier);
        }
      }
      /* Rule::Divert => {
        if let Some(divert) = parse_divert(inner_token) {
          settings.divert.push(divert);
        }
      } */
      _ => {}
    }
  }
}
fn add_command_to_block(token: Pair<Rule>, block: &mut Block) {
  match block {
    Block::Text { id: _, settings } => {
      add_command_to_settings(token, settings);
    }
    Block::Choice { id: _, settings } => {
      add_command_to_settings(token, settings);
    }
    _ => {}
  }
}
/*
fn parse_divert(token: Pair<Rule>) -> Option<Divert> {
  if token.as_rule() != Rule::Divert {
    return None;
  }
  //Divert = { "->"  ~ " "* ~ Identifier ~ ("." ~ Identifier)? }

  let mut knot: Option<String> = None;
  let mut stitch: Option<String> = None;

  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Identifier {
      if knot.is_none() {
        knot = Some(inner_token.as_str().to_string());
      } else {
        stitch = Some(inner_token.as_str().to_string());
      }
    }
  }

  knot.as_ref()?;

  Some(Divert {
    knot: knot.unwrap(),
    stitch,
  })
} */

fn parse_modifier(token: Pair<Rule>) -> Option<Modifier> {
  if token.as_rule() != Rule::Modifier {
    return None;
  }
  //Modifier = { "mod" ~ " "+ ~ Identifier ~ " "+ ~ Value}

  let mut modifier = Modifier::default();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Identifier => {
        modifier.variable.id = inner_token.as_str().to_string();
        //TODO KIND
      }

      Rule::Value => {
        modifier.new_value = inner_token.as_str().to_string();
      }
      _ => {}
    }
  }
  Some(modifier)
}

fn parse_frequency(token: Pair<Rule>) -> Option<FrequencyModifier> {
  if token.as_rule() != Rule::Frequency {
    return None;
  }

  let mut frequency = FrequencyModifier::default();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Condition => {
        if let Some(condition) = parse_condition(inner_token) {
          frequency.condition = condition;
        }
      }

      Rule::Float | Rule::Integer => {
        let value = inner_token.as_str().parse::<f32>().unwrap();
        frequency.value = value;
      }
      _ => {}
    }
  }

  Some(frequency)
}

fn parse_requirement(token: Pair<Rule>) -> Option<Requirement> {
  if token.as_rule() != Rule::Requirement {
    return None;
  }

  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Condition {
      if let Some(condition) = parse_condition(inner_token) {
        return Some(Requirement { condition });
      }
    }
  }
  None
}

fn parse_condition(token: Pair<Rule>) -> Option<Condition> {
  if token.as_rule() != Rule::Condition {
    return None;
  }
  /*Condition = { Identifier ~ " "* ~ (ComparisonOperator ~ " "*)? ~ Value } */

  let mut condition = Condition::default();

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Identifier => {
        condition.variable.id = inner_token.as_str().to_string();
        //TODO KIND
      }
      Rule::ComparisonOperator => {
        if let Some(operator) = parse_comparison_operator(inner_token) {
          condition.operator = operator;
        }
      }
      Rule::Value => {
        condition.value = inner_token.as_str().to_string();
      }
      _ => {}
    }
  }
  Some(condition)
}

fn parse_comparison_operator(token: Pair<Rule>) -> Option<Operator> {
  if token.as_rule() != Rule::ComparisonOperator {
    return None;
  }

  match token.as_str() {
    "!=" => Some(Operator::NotEqual),
    "!" => Some(Operator::NotEqual),
    "=" => Some(Operator::Equal),
    "<=" => Some(Operator::LessOrEqualThan),
    ">=" => Some(Operator::GreaterOrEqualThan),
    "<" => Some(Operator::LessThan),
    ">" => Some(Operator::GreaterThan),
    _ => None,
  }
}

//TODO: Integer
fn parse_probability(token: Pair<Rule>) -> Option<f32> {
  if token.as_rule() != Rule::Probability {
    return None;
  }

  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Float {
      let value = inner_token.as_str().parse::<f32>().unwrap();

      return Some(value);
    }
    if inner_token.as_rule() == Rule::Percentage {
      if let Some(integer) = inner_token.into_inner().next() {
        let value = integer.as_str().parse::<u64>().unwrap();
        return Some(value as f32 / 100.);
      }
    }
  }

  None
}

#[cfg(test)]
mod test {
  use crate::*;
  use cuentitos_common::{
    Block, BlockSettings, Condition, FrequencyModifier, Modifier, Operator, Requirement, Variable,
  };
  use pest::iterators::Pair;
  use pest::Parser;
  use rand::distributions::Alphanumeric;
  use rand::{self, Rng};

  const SPECIAL_CHARACTERS: [&str; 11] = [".", "_", "/", ",", ";", "'", " ", "?", "!", "¿", "¡"];

  const COMPARISON_OPERATORS: [&str; 7] = ["=", "!=", "<", ">", "<=", ">=", "!"];

  const INDENTATIONS: [&str; 1] = ["  "];

  #[test]
  fn parse_file_from_path_correctly() {
    parse_database_from_path("../examples/story-example.cuentitos").unwrap();
    //TODO: compare with fixture
  }

  #[test]
  fn parse_file_correctly() {
    let unparsed_file = include_str!("../../examples/story-example.cuentitos");
    let token = short_parse(Rule::File, &unparsed_file);
    parse_file(token).unwrap();
    //TODO: compare with fixture
  }

  /*#[test]
    fn parse_blocks_correctly() {
      /*

      BlockBlock = {
          (NamedBucket | Choice | Text)  ~  " "* ~ Command* ~ " "* ~ (NEWLINE | EOI) ~ NewBlock*
      }

      */

      let float = rand::thread_rng().gen_range(i8::MIN as f32..i8::MAX as f32);
      let probability_string = format!("({})", float);

      let string = make_random_string();
      let child_string = make_random_string();

      let text_string = format!("{} {}", probability_string, string);

      let probability_token = short_parse(Rule::Probability, &probability_string);
      let probability = parse_probability(probability_token);

      let knot = make_random_identifier();
      let divert_string = format!("\n -> {}", knot);

      let blocks_string = format!("{}{}\n\t{}", text_string, divert_string, child_string);

      let expected_divert = Divert {
        knot: knot.clone(),
        stitch: None,
      };

      let child_blocks = Block {
        text: child_string,
        blocks_type: BlockType::Text,
        ..Default::default()
      };

      let expected_blocks = Block {
        text: string,
        probability: probability,
        blocks_type: BlockType::Text,
        divert: vec![expected_divert],
        blocks: vec![child_blocks],
        ..Default::default()
      };

      let blocks_token = short_parse(Rule::BlockBlock, &blocks_string);
      let blocks = parse_blocks(blocks_token).unwrap();

      assert_eq!(blocks, expected_blocks);
    }
    #[test]
    fn get_blocks_from_new_block_correctly() {
      let string = make_random_string();
      let new_block_string = format!("\t{}", string);

      let expected_blocks = Block {
        text: string,
        ..Default::default()
      };

      let new_block_token = short_parse(Rule::NewBlock, &new_block_string);
      let blocks_token = get_blocks_from_new_block(new_block_token);
      let blocks = parse_blocks(blocks_token[0].clone()).unwrap();

      assert_eq!(blocks, expected_blocks);
    }
    #[test]
    fn parse_named_bucket_correctly() {
      //NamedBucket = { "[" ~ " "* ~ Probability? ~ SnakeCase ~ " "* ~ "]" }

      let float = rand::thread_rng().gen_range(i8::MIN as f32..i8::MAX as f32);
      let probability_string = format!("({})", float);

      let snake_case = make_random_snake_case();

      let named_bucket_string = format!("[{} {}]", probability_string, snake_case);
      let token = short_parse(Rule::NamedBucket, &named_bucket_string);
      let named_bucket = parse_named_bucket(token).unwrap();

      let probability_token = short_parse(Rule::Probability, &probability_string);
      let probability = parse_probability(probability_token);

      let expected_value = Block {
        text: snake_case,
        probability: probability,
        blocks_type: BlockType::NamedBucket,
        ..Default::default()
      };
      assert_eq!(named_bucket, expected_value);
    }
  */
  #[test]
  fn parse_choice_correctly() {
    //Choice = { "*" ~ " "* ~ Probability? ~ String }

    let float = rand::thread_rng().gen_range(i8::MIN as f32..i8::MAX as f32);
    let probability_string = format!("({})", float);

    let string = make_random_string();

    let choice_string = format!("*{} {}", probability_string, string);
    let token = short_parse(Rule::Choice, &choice_string);
    let choice = parse_choice(token).unwrap();

    let probability_token = short_parse(Rule::Probability, &probability_string);
    let probability = parse_probability(probability_token);

    let expected_settings = BlockSettings {
      chance: probability,
      ..Default::default()
    };
    let expected_value = Block::Choice {
      id: string,
      settings: expected_settings,
    };
    assert_eq!(choice, expected_value);
  }

  #[test]
  fn parse_text_correctly() {
    //Text = { Probability? ~ String }

    let float = rand::thread_rng().gen_range(0 as f32..1 as f32);
    let probability_string = format!("({})", float);

    let string = make_random_string();

    let text_string = format!("{} {}", probability_string, string);
    let token = short_parse(Rule::Text, &text_string);
    let text = parse_text(token).unwrap();

    let probability_token = short_parse(Rule::Probability, &probability_string);
    let probability = parse_probability(probability_token);

    let expected_settings = BlockSettings {
      chance: probability,
      ..Default::default()
    };

    let expected_value = Block::Text {
      id: string,
      settings: expected_settings,
    };
    assert_eq!(text, expected_value);
  }

  #[test]
  fn command_gets_added_to_blocks_correctly() {
    let text_id = make_random_string();
    let mut block = Block::Text {
      id: text_id.clone(),
      settings: BlockSettings::default(),
    };

    let mut block_settings = BlockSettings::default();
    // //Command = {NEWLINE ~ (Indentation | " ")* ~ (Requirement | Frequency | Modifier | Divert) }

    //Modifier
    let identifier = make_random_identifier();
    let new_value = rand::thread_rng().gen::<f32>().to_string();
    let modifier_string = format!("\n mod {} {}", identifier, new_value);

    let expected_modifier = Modifier {
      variable: Variable {
        id: identifier,
        ..Default::default()
      },
      new_value,
    };

    block_settings.modifiers.push(expected_modifier);

    let token = short_parse(Rule::Command, &modifier_string);
    add_command_to_block(token, &mut block);

    /*  //Divert
    let knot = make_random_identifier();
    let divert_string = format!("\n -> {}", knot);

    let expected_divert = Divert {
      knot: knot.clone(),
      stitch: None,
    };
    expected_blocks.divert.push(expected_divert);

    let token = short_parse(Rule::Command, &divert_string);
    add_command_to_blocks(token, &mut blocks); */

    //Frequency

    let condition_string = make_random_condition();
    let condition_token = short_parse(Rule::Condition, &condition_string);
    let condition = parse_condition(condition_token).unwrap();

    let change_value: f32 = rand::thread_rng().gen();
    let frequency_string = format!("\n freq {} {}", condition_string, change_value);
    let expected_frequency = FrequencyModifier {
      condition,
      value: change_value,
    };

    block_settings.frequency_modifiers.push(expected_frequency);

    let token = short_parse(Rule::Command, &frequency_string);
    add_command_to_block(token, &mut block);

    //Requirement

    let condition_string = make_random_condition();
    let condition_token = short_parse(Rule::Condition, &condition_string);
    let condition = parse_condition(condition_token).unwrap();

    let requirement_string = format!("\n req {}", condition_string);
    let expected_requirement = Requirement { condition };

    block_settings.requirements.push(expected_requirement);

    let token = short_parse(Rule::Command, &requirement_string);
    add_command_to_block(token, &mut block);

    let expected_block = Block::Text {
      id: text_id,
      settings: block_settings,
    };

    assert_eq!(block, expected_block);
  }
  /*
  #[test]
  fn parse_divert_correctly() {
    //Divert = { "->"  ~ " "* ~ Identifier ~ ("." ~ Identifier)? }
    let knot = make_random_identifier();
    let divert_string = format!("-> {}", knot);

    let expected_value = Divert {
      knot: knot.clone(),
      stitch: None,
    };

    let token = short_parse(Rule::Divert, &divert_string);
    let divert = parse_divert(token).unwrap();

    assert_eq!(divert, expected_value);

    let stitch = make_random_identifier();

    let divert_string = format!("-> {}.{}", knot, stitch);

    let expected_value = Divert {
      knot,
      stitch: Some(stitch),
    };

    let token = short_parse(Rule::Divert, &divert_string);
    let divert = parse_divert(token).unwrap();

    assert_eq!(divert, expected_value);
  } */

  #[test]
  fn parse_modifier_correctly() {
    //Modifier = { "mod" ~ " "+ ~ Identifier ~ " "+ ~ Value}
    let identifier = make_random_identifier();
    let new_value = rand::thread_rng().gen::<f32>().to_string();
    let modifier_string = format!("mod {} {}", identifier, new_value);

    let expected_value = Modifier {
      variable: Variable {
        id: identifier,
        ..Default::default()
      },
      new_value,
    };

    let token = short_parse(Rule::Modifier, &modifier_string);
    let modifier = parse_modifier(token).unwrap();

    assert_eq!(modifier, expected_value);
  }

  #[test]
  fn parse_frequency_correctly() {
    //Frequency = { "freq" ~ " "+ ~ Condition ~ " "+ ~ ( Float | Integer ) }
    let condition_string = make_random_condition();
    let condition_token = short_parse(Rule::Condition, &condition_string);
    let condition = parse_condition(condition_token).unwrap();

    let change_value: f32 = rand::thread_rng().gen();
    let frequency_string = format!("freq {} {}", condition_string, change_value);
    let expected_value = FrequencyModifier {
      condition,
      value: change_value,
    };

    let token = short_parse(Rule::Frequency, &frequency_string);
    let frequency = parse_frequency(token).unwrap();

    assert_eq!(frequency, expected_value);
  }

  #[test]
  fn parse_requirement_correctly() {
    //Requirement = { "req" ~ " "+ ~ Condition }
    let condition_string = make_random_condition();
    let condition_token = short_parse(Rule::Condition, &condition_string);
    let condition = parse_condition(condition_token).unwrap();

    let requirement_string = format!("req {}", condition_string);
    let expected_value = Requirement { condition };

    let token = short_parse(Rule::Requirement, &requirement_string);
    let requirement = parse_requirement(token).unwrap();

    assert_eq!(requirement, expected_value);
  }

  #[test]
  fn parse_condition_correctly() {
    /*Condition = { Identifier ~ " "* ~ (ComparisonOperator ~ " "*)? ~ Value } */
    let identifier = make_random_identifier();

    let operator_string =
      COMPARISON_OPERATORS[rand::thread_rng().gen_range(0..COMPARISON_OPERATORS.len())];
    let operator_token = short_parse(Rule::ComparisonOperator, operator_string);
    let operator = parse_comparison_operator(operator_token).unwrap();

    let value: f32 = rand::thread_rng().gen();

    let condition_string = format!("{} {} {}", identifier, operator_string, value);

    let expected_value = Condition {
      variable: Variable {
        id: identifier,
        ..Default::default()
      },
      operator: operator,
      value: value.to_string(),
    };

    let token = short_parse(Rule::Condition, &condition_string);
    let condition = parse_condition(token).unwrap();

    assert_eq!(condition, expected_value);
  }

  #[test]
  fn parse_operators_correctly() {
    let token = short_parse(Rule::ComparisonOperator, "=");
    assert_eq!(parse_comparison_operator(token).unwrap(), Operator::Equal);

    let token = short_parse(Rule::ComparisonOperator, "!=");
    assert_eq!(
      parse_comparison_operator(token).unwrap(),
      Operator::NotEqual
    );

    let token = short_parse(Rule::ComparisonOperator, "<");
    assert_eq!(
      parse_comparison_operator(token).unwrap(),
      Operator::LessThan
    );

    let token = short_parse(Rule::ComparisonOperator, ">");
    assert_eq!(
      parse_comparison_operator(token).unwrap(),
      Operator::GreaterThan
    );

    let token = short_parse(Rule::ComparisonOperator, "<=");
    assert_eq!(
      parse_comparison_operator(token).unwrap(),
      Operator::LessOrEqualThan
    );

    let token = short_parse(Rule::ComparisonOperator, ">=");
    assert_eq!(
      parse_comparison_operator(token).unwrap(),
      Operator::GreaterOrEqualThan
    );

    let token = short_parse(Rule::ComparisonOperator, "!");
    assert_eq!(
      parse_comparison_operator(token).unwrap(),
      Operator::NotEqual
    );
  }
  /* #[test]
   fn percentage_probability_parse_correctly() {
     //probability = { "(" ~ " "* ~ ( percentage | float | integer ) ~ " "* ~ ")" ~ " "* }
     let percentage = rand::thread_rng().gen_range(u8::MIN..u8::MAX);
     let expected_value: PercentageProbability = PercentageProbability { value: percentage };

     let probability_string = format!("({}%)", percentage);

     let token = short_parse(Rule::Probability, &probability_string);

     let probability = parse_probability(token).unwrap();
     let probability = probability
       .as_any()
       .downcast_ref::<PercentageProbability>()
       .unwrap();

     assert_eq!(*probability, expected_value);
   }

   #[test]
   fn float_probability_parse_correctly() {
     let float = rand::thread_rng().gen_range(i8::MIN as f32..i8::MAX as f32);
     let expected_value = FloatProbability { value: float };

     let probability_string = format!("({})", float);

     let token = PalabritasParser::parse(Rule::Probability, &probability_string)
       .expect("unsuccessful parse")
       .next()
       .unwrap();

     let probability = parse_probability(token).unwrap();
     let probability = probability
       .as_any()
       .downcast_ref::<FloatProbability>()
       .unwrap();

     assert_eq!(*probability, expected_value);
   }
  */
  #[test]
  fn parse_char_rule() {
    //char = { ASCII_ALPHANUMERIC | "." | "_" | "/" | "," | ";" | "'" | " " | "?" | "!" | "¿" | "¡"}
    let alphanumeric_char = (rand::thread_rng().sample(&Alphanumeric) as char).to_string();
    assert_parse_rule(Rule::Char, &alphanumeric_char);

    for special_character in SPECIAL_CHARACTERS {
      assert_parse_rule(Rule::Char, special_character);
    }
  }

  #[test]
  fn parse_integer_rule() {
    //integer = { "-"? ~ ASCII_DIGIT+ }
    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    assert_parse_rule(Rule::Integer, &integer);
  }

  #[test]
  fn parse_float_rule() {
    //float = { integer ~ "." ~ ASCII_DIGIT* }
    let float = rand::thread_rng()
      .gen_range(i8::MIN as f32..i8::MAX as f32)
      .to_string();
    assert_parse_rule(Rule::Float, &float);
  }

  #[test]
  fn parse_percentage_rule() {
    //percentage = { integer ~ "%" }
    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse_rule(Rule::Percentage, &percentage);
  }

  #[test]
  fn parse_indentation_rule() {
    //indentation = { "  }
    for indentation in INDENTATIONS {
      assert_parse_rule(Rule::Indentation, &indentation);
    }
  }

  #[test]
  fn parse_string_rule() {
    //string = { char+ }
    assert_parse_rule(Rule::String, &make_random_string());
  }

  #[test]
  fn parse_comparison_operator_rule() {
    //comparison_operator = { "!=" | "=" | "<=" | ">=" | "<" | ">" | "!" }
    for operator in COMPARISON_OPERATORS {
      assert_parse_rule(Rule::ComparisonOperator, operator);
    }
  }

  #[test]
  fn parse_snake_case_rule() {
    //snake_case = { ASCII_ALPHA_LOWER ~ (ASCII_ALPHA_LOWER | "_" | ASCII_DIGIT)* }
    assert_parse_rule(Rule::SnakeCase, &make_random_snake_case());
  }

  #[test]
  fn parse_identifier_rule() {
    //identifier = { (ASCII_ALPHA | "_" ) ~ (ASCII_ALPHANUMERIC | "_")* }
    assert_parse_rule(Rule::Identifier, &make_random_identifier());
  }

  #[test]
  fn parse_value_rule() {
    //value = { identifier | float | percentage | integer}
    let identifier = make_random_identifier();
    assert_parse_rule(Rule::Value, &identifier);

    let float = rand::thread_rng()
      .gen_range(i8::MIN as f32..i8::MAX as f32)
      .to_string();
    assert_parse_rule(Rule::Value, &float);

    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse_rule(Rule::Value, &percentage);

    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    assert_parse_rule(Rule::Value, &integer);
  }

  #[test]
  fn parse_condition_rule() {
    //condition = { identifier ~ " "* ~ (comparison_operator ~ " "*)? ~ value }
    let identifier = make_random_identifier();
    let comparison_operator =
      COMPARISON_OPERATORS[rand::thread_rng().gen_range(0..COMPARISON_OPERATORS.len())];
    let value = make_random_identifier();

    assert_parse_rule(Rule::Condition, &(identifier.clone() + " " + &value));
    assert_parse_rule(
      Rule::Condition,
      &(identifier + comparison_operator + &value),
    );
  }

  #[test]
  fn parse_requirement_rule() {
    //requirement = { "req" ~ " "+ ~ condition }
    let condition = make_random_condition();
    assert_parse_rule(Rule::Requirement, &("req ".to_string() + &condition));
  }
  #[test]
  fn parse_frequency_rule() {
    //frequency = { "freq" ~ " "+ ~ condition ~ " "+ ~ ( float | integer ) }
    let condition = make_random_condition();
    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    let float = rand::thread_rng()
      .gen_range(i8::MIN as f32..i8::MAX as f32)
      .to_string();

    assert_parse_rule(
      Rule::Frequency,
      &("freq ".to_string() + &condition + " " + &integer),
    );
    assert_parse_rule(
      Rule::Frequency,
      &("freq ".to_string() + &condition + " " + &float),
    );
  }

  #[test]
  fn parse_modifier_rule() {
    //modifier = { "mod" ~ " "+ ~ identifier ~ " "+ ~ value}
    let identifier = make_random_identifier();
    let value = make_random_identifier();

    assert_parse_rule(
      Rule::Modifier,
      &("mod ".to_string() + &identifier + " " + &value),
    );
  }

  #[test]
  fn parse_divert_rule() {
    //divert = { "->"  ~ " "* ~ identifier ~ ("." ~ identifier)? }
    let knot = make_random_identifier();
    let stitch = make_random_identifier();

    assert_parse_rule(Rule::Divert, &("->".to_string() + &knot));
    assert_parse_rule(Rule::Divert, &("->".to_string() + &knot + "." + &stitch));
  }

  #[test]
  fn parse_command_rule() {
    //Command = {NEWLINE ~ Indentation* ~ (Requirement | Frequency | Modifier | Divert) }
    let requirement = "\nreq ".to_string() + &(make_random_condition());
    assert_parse_rule(Rule::Command, &(requirement));

    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    let frequency = "\nfreq ".to_string() + &make_random_condition() + " " + &integer;
    assert_parse_rule(Rule::Command, &(frequency));

    let modifier =
      "\nmod ".to_string() + &make_random_identifier() + " " + &make_random_identifier();
    assert_parse_rule(Rule::Command, &(modifier));

    let divert = "\n->".to_string() + &make_random_identifier();
    assert_parse_rule(Rule::Command, &(divert));
  }

  #[test]
  fn parse_probability_rule() {
    //probability = { "(" ~ " "* ~ ( percentage | float | integer ) ~ " "* ~ ")" ~ " "* }
    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse_rule(Rule::Probability, &("(".to_string() + &percentage + ")"));

    let float = rand::thread_rng()
      .gen_range(i8::MIN as f32..i8::MAX as f32)
      .to_string();
    assert_parse_rule(Rule::Probability, &("(".to_string() + &float + ")"));

    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    assert_parse_rule(Rule::Probability, &("(".to_string() + &integer + ")"));
  }

  #[test]
  fn parse_section_rule() {
    //Knot = {"===" ~ " "* ~ Identifier ~ " "* ~"===" ~ " "* ~ NEWLINE ~ ( NEWLINE | BlockBlock | Stitch | NamedBucket )* }
    let identifier = make_random_identifier();
    assert_parse_rule(Rule::Section, &("#".to_string() + &identifier + "\n"));
  }

  #[test]
  fn parse_subsection_rule() {
    //stitch = {"=" ~ " "* ~ identifier ~ " "*}
    let identifier = make_random_identifier();
    assert_parse_rule(Rule::Subsection, &("##".to_string() + &identifier));
  }

  #[test]
  fn parse_text_rule() {
    //text = { probability? ~ string }
    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    let probability = "(".to_string() + &integer + ")";
    assert_parse_rule(Rule::Text, &make_random_string());
    assert_parse_rule(Rule::Text, &(probability + &make_random_string()));
  }

  #[test]
  fn parse_choice_rule() {
    //choice = { "*" ~ " "* ~ text }
    let text = make_random_string();
    assert_parse_rule(Rule::Choice, &("*".to_string() + &text));
  }

  #[test]
  fn parse_named_bucket_rule() {
    //named_bucket = { "[" ~ " "* ~ probability? ~ snake_case ~ " "* ~ "]"}
    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    let probability = "(".to_string() + &integer + ")";

    assert_parse_rule(
      Rule::NamedBucket,
      &("[".to_string() + &make_random_snake_case() + "]"),
    );

    assert_parse_rule(
      Rule::NamedBucket,
      &("[".to_string() + &probability + &make_random_snake_case() + "]"),
    );
  }

  /* #[test]
   fn parse_block_blocks_rule() {
      //BlockBlock = {
      // (choice | Text)  ~  " "* ~ Command* ~ " "* ~ (NEWLINE | EOI) ~ NewBlock*
      //}

      let choice = "*".to_string() + &make_random_string();
      assert_parse_rule(Rule::BlockBlock, &choice);

      let text = make_random_string();
      assert_parse_rule(Rule::BlockBlock, &text);

      let new_block = "\n  ".to_string() + &make_random_string();
      assert_parse_rule(Rule::BlockBlock, &(text + &new_block));
    }
  */
  #[test]
  fn parse_file_rule() {
    //File = { SOI ~ (NEWLINE | BlockBlock | Knot )* ~ EOI }
    let unparsed_file = include_str!("../../examples/story-example.cuentitos");
    assert_parse_rule(Rule::File, &unparsed_file);
  }

  fn assert_parse_rule(rule: Rule, input: &str) {
    let pair = PalabritasParser::parse(rule, input)
      .expect("unsuccessful parse")
      .next()
      .unwrap();
    assert_eq!(pair.as_rule(), rule);
    assert_eq!(pair.as_str(), input);
  }

  fn make_random_condition() -> String {
    make_random_identifier() + " " + &make_random_identifier()
  }

  fn make_random_snake_case() -> String {
    let alphanumeric_size = rand::thread_rng().gen_range(1..20);
    let underscore_size = rand::thread_rng().gen_range(1..5);

    //Making alphanumeric string
    let snake_case: Vec<u8> = rand::thread_rng()
      .sample_iter(&Alphanumeric)
      .take(alphanumeric_size)
      .collect();

    let mut snake_case = std::str::from_utf8(&snake_case).unwrap().to_string();

    //Adding underscores
    for _ in 0..underscore_size {
      snake_case += "_";
    }
    shuffle_string(&mut snake_case);

    //Making sure starting character is not a number
    let mut starting_char = rand::thread_rng().sample(&Alphanumeric) as char;
    while starting_char.is_numeric() {
      starting_char = rand::thread_rng().sample(&Alphanumeric) as char;
    }

    snake_case = starting_char.to_string() + &snake_case;

    snake_case.to_lowercase()
  }

  fn make_random_identifier() -> String {
    let alphanumeric_size = rand::thread_rng().gen_range(1..20);
    let underscore_size = rand::thread_rng().gen_range(1..5);

    //Making alphanumeric string
    let identifier: Vec<u8> = rand::thread_rng()
      .sample_iter(&Alphanumeric)
      .take(alphanumeric_size)
      .collect();

    let mut identifier = std::str::from_utf8(&identifier).unwrap().to_string();

    //Adding underscores
    for _ in 0..underscore_size {
      identifier += "_";
    }
    shuffle_string(&mut identifier);

    //Making sure starting character is not a number
    let mut starting_char = rand::thread_rng().sample(&Alphanumeric) as char;
    while starting_char.is_numeric() {
      starting_char = rand::thread_rng().sample(&Alphanumeric) as char;
    }

    identifier = starting_char.to_string() + &identifier;

    identifier
  }

  fn make_random_string() -> String {
    let alphanumeric_size = rand::thread_rng().gen_range(1..20);
    let special_characters_size = rand::thread_rng().gen_range(1..20);

    //Making alphanumeric string
    let string: Vec<u8> = rand::thread_rng()
      .sample_iter(&Alphanumeric)
      .take(alphanumeric_size)
      .collect();

    let mut string = std::str::from_utf8(&string).unwrap().to_string();

    //Adding special characters
    for _ in 0..special_characters_size {
      string += SPECIAL_CHARACTERS[rand::thread_rng().gen_range(0..SPECIAL_CHARACTERS.len())];
    }

    shuffle_string(&mut string);
    string.trim().to_string()
  }

  fn shuffle_string(string: &mut String) {
    //Shuffling characters
    let mut final_string = String::default();

    while !string.is_empty() {
      let mut index = rand::thread_rng().gen_range(0..string.len());
      while !string.is_char_boundary(index) {
        index -= 1;
      }
      final_string += &string.remove(index).to_string();
    }

    *string = final_string;
  }

  fn short_parse(rule: Rule, input: &str) -> Pair<Rule> {
    PalabritasParser::parse(rule, input)
      .expect("unsuccessful parse")
      .next()
      .unwrap()
  }
}
