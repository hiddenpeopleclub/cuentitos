extern crate pest;
#[macro_use]
extern crate pest_derive;

use palabritas_common::{
  Condition, Content, Divert, File, FloatProbability, Frequency, Modifier, Operator, Percentage,
  Probability, Requirement,
};
use pest::{iterators::Pair, Parser};

#[derive(Parser)]
#[grammar = "palabritas.pest"]
pub struct PalabritasParser;

fn main() {
  let unparsed_file = include_str!("../../examples/story-example.cuentitos");
  PalabritasParser::parse(Rule::File, unparsed_file)
    .expect("unsuccessful parse") // unwrap the parse result
    .next()
    .unwrap();
}

fn parse_file(token: Pair<Rule>) -> Option<File> {
  if token.as_rule() != Rule::File {
    return None;
  }

  let mut file = File::default();

  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::BlockContent {
      if let Some(content) = parse_block_content(inner_token) {
        file.content.push(content);
      }
    }
  }

  Some(file)
}

fn parse_block_content(token: Pair<Rule>) -> Option<Content> {
  if token.as_rule() != Rule::BlockContent {
    return None;
  }
  //    (NamedBucket | Choice | Text)  ~  " "* ~ Command* ~ " "* ~ (NEWLINE | EOI) ~ NewBlock*
  let mut content = Content::default();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Text => {
        if let Some(text) = parse_text(inner_token) {
          content = text;
        }
      }
      Rule::NamedBucket => {
        println!("NamedBucket: {}", inner_token.as_str())
      }
      Rule::Choice => {
        println!("Choice: {}", inner_token.as_str())
      }
      Rule::Command => {
        add_command_to_content(inner_token, &mut content);
      }
      Rule::NewBlock => {
        if let Some(inner_content) = parse_block_content(inner_token) {
          content.content.push(inner_content);
        }
      }

      _ => {}
    }
  }

  Some(content)
}

fn parse_text(token: Pair<Rule>) -> Option<Content> {
  if token.as_rule() != Rule::Text {
    return None;
  }

  let mut content = Content::default();

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Probability => {
        content.probability = parse_probability(inner_token);
      }
      Rule::String => {
        content.text = inner_token.as_str().to_string();
      }
      _ => {}
    }
  }

  None
}

fn add_command_to_content(token: Pair<Rule>, content: &mut Content) {
  if token.as_rule() != Rule::Command {
    return;
  }

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      //Command = {NEWLINE ~ (Indentation | " ")* ~ (Requirement | Frequency | Modifier | Divert) }
      Rule::Requirement => {
        if let Some(requirement) = parse_requirement(inner_token) {
          content.requirements.push(requirement);
        }
      }
      Rule::Frequency => {
        if let Some(frequency) = parse_frequency(inner_token) {
          content.frequency_changes.push(frequency);
        }
      }
      Rule::Modifier => {
        if let Some(modifier) = parse_modifier(inner_token) {
          content.modifiers.push(modifier);
        }
      }
      Rule::Divert => {
        if let Some(divert) = parse_divert(inner_token) {
          content.divert.push(divert);
        }
      }
      _ => {}
    }
  }
}

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
}

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

fn parse_frequency(token: Pair<Rule>) -> Option<Frequency> {
  if token.as_rule() != Rule::Frequency {
    return None;
  }

  let mut frequency = Frequency::default();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Condition => {
        if let Some(condition) = parse_condition(inner_token) {
          frequency.condition = condition;
        }
      }

      Rule::Float | Rule::Integer => {
        let value = inner_token.as_str().parse::<f32>().unwrap();
        frequency.change_value = value;
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

fn parse_probability(token: Pair<Rule>) -> Option<Box<dyn Probability>> {
  if token.as_rule() != Rule::Probability {
    return None;
  }

  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Float {
      let value = inner_token.as_str().parse::<f32>().unwrap();
      let probability = FloatProbability { value };

      return Some(Box::new(probability));
    }
    if inner_token.as_rule() == Rule::Percentage {
      let value = inner_token.as_str().parse::<u8>().unwrap();
      let percentage = Percentage { value };

      return Some(Box::new(percentage));
    }
  }

  None
}

#[cfg(test)]
mod test {

  use pest::Parser;
  use rand::distributions::Alphanumeric;
  use rand::{self, Rng};

  #[derive(Parser)]
  #[grammar = "palabritas.pest"]
  pub struct PalabritasParser;

  const SPECIAL_CHARACTERS: [&str; 11] = [".", "_", "/", ",", ";", "'", " ", "?", "!", "¿", "¡"];

  const COMPARISON_OPERATORS: [&str; 7] = ["=", "!=", "<", ">", "<=", ">=", "!"];

  const INDENTATIONS: [&str; 2] = ["  ", "\t"];

  #[test]
  fn parse_char() {
    //char = { ASCII_ALPHANUMERIC | "." | "_" | "/" | "," | ";" | "'" | " " | "?" | "!" | "¿" | "¡"}
    let alphanumeric_char = (rand::thread_rng().sample(&Alphanumeric) as char).to_string();
    assert_parse(Rule::Char, &alphanumeric_char);

    for special_character in SPECIAL_CHARACTERS {
      assert_parse(Rule::Char, special_character);
    }
  }

  #[test]
  fn parse_integer() {
    //integer = { "-"? ~ ASCII_DIGIT+ }
    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    assert_parse(Rule::Integer, &integer);
  }

  #[test]
  fn parse_float() {
    //float = { integer ~ "." ~ ASCII_DIGIT* }
    let float = rand::thread_rng()
      .gen_range(i8::MIN as f32..i8::MAX as f32)
      .to_string();
    assert_parse(Rule::Float, &float);
  }

  #[test]
  fn parse_percentage() {
    //percentage = { integer ~ "%" }
    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse(Rule::Percentage, &percentage);
  }

  #[test]
  fn parse_indentation() {
    //indentation = { "  " | "\t" }
    for indentation in INDENTATIONS {
      assert_parse(Rule::Indentation, &indentation);
    }
  }

  #[test]
  fn parse_string() {
    //string = { char+ }
    assert_parse(Rule::String, &make_random_string());
  }

  #[test]
  fn parse_comparison_operator() {
    //comparison_operator = { "!=" | "=" | "<=" | ">=" | "<" | ">" | "!" }
    for operator in COMPARISON_OPERATORS {
      assert_parse(Rule::ComparisonOperator, operator);
    }
  }

  #[test]
  fn parse_snake_case() {
    //snake_case = { ASCII_ALPHA_LOWER ~ (ASCII_ALPHA_LOWER | "_" | ASCII_DIGIT)* }
    assert_parse(Rule::SnakeCase, &make_random_snake_case());
  }

  #[test]
  fn parse_identifier() {
    //identifier = { (ASCII_ALPHA | "_" ) ~ (ASCII_ALPHANUMERIC | "_")* }
    assert_parse(Rule::Identifier, &make_random_identifier());
  }

  #[test]
  fn parse_value() {
    //value = { identifier | float | percentage | integer}
    let identifier = make_random_identifier();
    assert_parse(Rule::Value, &identifier);

    let float = rand::thread_rng()
      .gen_range(i8::MIN as f32..i8::MAX as f32)
      .to_string();
    assert_parse(Rule::Value, &float);

    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse(Rule::Value, &percentage);

    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    assert_parse(Rule::Value, &integer);
  }

  #[test]
  fn parse_condition() {
    //condition = { identifier ~ " "* ~ (comparison_operator ~ " "*)? ~ value }
    let identifier = make_random_identifier();
    let comparison_operator =
      COMPARISON_OPERATORS[rand::thread_rng().gen_range(0..COMPARISON_OPERATORS.len())];
    let value = make_random_identifier();

    assert_parse(Rule::Condition, &(identifier.clone() + " " + &value));
    assert_parse(
      Rule::Condition,
      &(identifier + comparison_operator + &value),
    );
  }

  #[test]
  fn parse_requirement() {
    //requirement = { "req" ~ " "+ ~ condition }
    let condition = make_random_condition();
    assert_parse(Rule::Requirement, &("req ".to_string() + &condition));
  }
  #[test]
  fn parse_frequency() {
    //frequency = { "freq" ~ " "+ ~ condition ~ " "+ ~ ( float | integer ) }
    let condition = make_random_condition();
    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    let float = rand::thread_rng()
      .gen_range(i8::MIN as f32..i8::MAX as f32)
      .to_string();

    assert_parse(
      Rule::Frequency,
      &("freq ".to_string() + &condition + " " + &integer),
    );
    assert_parse(
      Rule::Frequency,
      &("freq ".to_string() + &condition + " " + &float),
    );
  }

  #[test]
  fn parse_modifier() {
    //modifier = { "mod" ~ " "+ ~ identifier ~ " "+ ~ value}
    let identifier = make_random_identifier();
    let value = make_random_identifier();

    assert_parse(
      Rule::Modifier,
      &("mod ".to_string() + &identifier + " " + &value),
    );
  }

  #[test]
  fn parse_divert() {
    //divert = { "->"  ~ " "* ~ identifier ~ ("." ~ identifier)? }
    let knot = make_random_identifier();
    let stitch = make_random_identifier();

    assert_parse(Rule::Divert, &("->".to_string() + &knot));
    assert_parse(Rule::Divert, &("->".to_string() + &knot + "." + &stitch));
  }

  #[test]
  fn parse_command() {
    //Command = {NEWLINE ~ Indentation* ~ (Requirement | Frequency | Modifier | Divert) }
    let requirement = "\nreq ".to_string() + &(make_random_condition());
    assert_parse(Rule::Command, &(requirement));

    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    let frequency = "\nfreq ".to_string() + &make_random_condition() + " " + &integer;
    assert_parse(Rule::Command, &(frequency));

    let modifier =
      "\nmod ".to_string() + &make_random_identifier() + " " + &make_random_identifier();
    assert_parse(Rule::Command, &(modifier));

    let divert = "\n->".to_string() + &make_random_identifier();
    assert_parse(Rule::Command, &(divert));
  }

  #[test]
  fn parse_probability() {
    //probability = { "(" ~ " "* ~ ( percentage | float | integer ) ~ " "* ~ ")" ~ " "* }
    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse(Rule::Probability, &("(".to_string() + &percentage + ")"));

    let float = rand::thread_rng()
      .gen_range(i8::MIN as f32..i8::MAX as f32)
      .to_string();
    assert_parse(Rule::Probability, &("(".to_string() + &float + ")"));

    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    assert_parse(Rule::Probability, &("(".to_string() + &integer + ")"));
  }

  #[test]
  fn parse_knot() {
    //Knot = {"===" ~ " "* ~ Identifier ~ " "* ~"===" ~ " "* ~ NEWLINE ~ ( NEWLINE | BlockContent | Stitch | NamedBucket )* }
    let identifier = make_random_identifier();
    assert_parse(Rule::Knot, &("===".to_string() + &identifier + "===\n"));
  }

  #[test]
  fn parse_stitch() {
    //stitch = {"=" ~ " "* ~ identifier ~ " "*}
    let identifier = make_random_identifier();
    assert_parse(Rule::Stitch, &("=".to_string() + &identifier));
  }

  #[test]
  fn parse_text() {
    //text = { probability? ~ string }
    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    let probability = "(".to_string() + &integer + ")";
    assert_parse(Rule::Text, &make_random_string());
    assert_parse(Rule::Text, &(probability + &make_random_string()));
  }

  #[test]
  fn parse_choice() {
    //choice = { "*" ~ " "* ~ text }
    let text = make_random_string();
    assert_parse(Rule::Choice, &("*".to_string() + &text));
  }

  #[test]
  fn parse_named_bucket() {
    //named_bucket = { "[" ~ " "* ~ probability? ~ snake_case ~ " "* ~ "]"}
    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    let probability = "(".to_string() + &integer + ")";

    assert_parse(
      Rule::NamedBucket,
      &("[".to_string() + &make_random_snake_case() + "]"),
    );

    assert_parse(
      Rule::NamedBucket,
      &("[".to_string() + &probability + &make_random_snake_case() + "]"),
    );
  }

  #[test]
  fn parse_block_content() {
    //BlockContent = {
    // (choice | Text)  ~  " "* ~ Command* ~ " "* ~ (NEWLINE | EOI) ~ NewBlock*
    //}

    let choice = "*".to_string() + &make_random_string();
    assert_parse(Rule::BlockContent, &choice);

    let text = make_random_string();
    assert_parse(Rule::BlockContent, &text);

    let new_block = "\n  ".to_string() + &make_random_string();
    assert_parse(Rule::BlockContent, &(text + &new_block));
  }

  #[test]
  fn parse_file() {
    //File = { SOI ~ (NEWLINE | BlockContent | Knot )* ~ EOI }
    let unparsed_file = include_str!("../../examples/story-example.cuentitos");
    assert_parse(Rule::File, &unparsed_file);
  }

  fn assert_parse(rule: Rule, input: &str) {
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
    string
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
}
