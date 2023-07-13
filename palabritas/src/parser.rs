extern crate pest;
use std::collections::HashMap;
use std::path::Path;
use std::vec;

use cuentitos_common::condition::ComparisonOperator;
use cuentitos_common::modifier::ModifierOperator;
use cuentitos_common::{
  Block, BlockId, BlockSettings, Chance, Condition, Config, Database, FrequencyModifier, Function,
  I18n, Modifier, NextBlock, Requirement, Script, Section, SectionName,
};
use pest::{iterators::Pair, Parser};

use pest::error::LineColLocation;

use crate::error::PalabritasError;

#[derive(Parser)]
#[grammar = "palabritas.pest"]
struct PalabritasParser;

#[derive(Default)]
struct ParsingData {
  pub blocks: Vec<Vec<Block>>,
  pub i18n: I18n,
  pub config: Config,
  pub file: String,
  pub current_section: Option<Section>,
}

impl ParsingData {
  fn new(config: Config, file: String) -> Self {
    ParsingData {
      blocks: Vec::default(),
      i18n: I18n::from_config(&config),
      config,
      file,
      current_section: None,
    }
  }
}

pub fn parse_database_from_path<P>(path: P) -> Result<Database, PalabritasError>
where
  P: AsRef<Path>,
{
  if !path.as_ref().exists() {
    return Err(PalabritasError::PathDoesntExist(
      path.as_ref().to_path_buf(),
    ));
  }
  let story_path = match path.as_ref().is_file() {
    true => path.as_ref().to_path_buf(),
    false => {
      //TODO: search for it instead
      return Err(PalabritasError::PathIsNotAFile(path.as_ref().to_path_buf()));
    }
  };
  let mut config_path = story_path.clone();
  config_path.pop();
  let config = match Config::load(&config_path) {
    Ok(config) => config,
    Err(err) => {
      return Err(PalabritasError::CantReadFile {
        path: config_path,
        message: err.to_string(),
      });
    }
  };

  let str = match std::fs::read_to_string(&story_path) {
    Ok(str) => str,
    Err(e) => {
      return Err(PalabritasError::CantReadFile {
        path: story_path,
        message: e.to_string(),
      });
    }
  };

  match PalabritasParser::parse(Rule::Database, &str) {
    Ok(mut result) => {
      let token = result.next().unwrap();
      let file = story_path.display().to_string();
      let parsing_data = ParsingData::new(config, file);

      match parse_database(token, parsing_data) {
        Ok(database) => Ok(database),
        Err(error) => Err(error),
      }
    }
    Err(error) => {
      let (line, col) = match error.line_col {
        LineColLocation::Pos(line_col) => line_col,
        LineColLocation::Span(start, _) => (start.0, start.1),
      };

      Err(PalabritasError::ParseError {
        reason: error.to_string(),
        script: Script {
          line,
          col,
          file: story_path.display().to_string(),
        },
      })
    }
  }
}

fn parse_database(
  token: Pair<Rule>,
  mut parsing_data: ParsingData,
) -> Result<Database, PalabritasError> {
  fn add_section(
    name: &str,
    children: &Vec<BlockId>,
    parent: Option<Box<Section>>,
    block_id: BlockId,
    section_map: &mut HashMap<Section, usize>,
    blocks: &Vec<Block>,
  ) {
    let section_key = Section {
      name: name.to_string(),
      parent,
    };

    section_map.insert(section_key.clone(), block_id);

    for child in children {
      if let Block::Section { id: name, settings } = &blocks[*child] {
        add_section(
          name,
          &settings.children,
          Some(Box::new(section_key.clone())),
          *child,
          section_map,
          blocks,
        );
      }
    }
  }

  match_rule(&token, Rule::Database, &parsing_data.file)?;

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Block => {
        parse_block(inner_token, &mut parsing_data, 0)?;
      }
      Rule::Section => {
        parse_section(inner_token, &mut parsing_data, 0)?;
      }
      _ => {}
    }
  }

  if parsing_data.blocks.is_empty() {
    return Err(PalabritasError::FileIsEmpty);
  }

  let end_block = Block::Divert {
    next: NextBlock::EndOfFile,
    settings: BlockSettings {
      script: parsing_data.blocks[0]
        .last()
        .unwrap()
        .get_settings()
        .script
        .clone(),
      ..Default::default()
    },
  };
  parsing_data.blocks[0].push(end_block);

  let level_0_size = parsing_data.blocks[0].len();
  let mut ordered_blocks = Vec::default();

  for child_level in 0..parsing_data.blocks.len() {
    let mut index_offset = 0;
    for childs_in_level in parsing_data.blocks.iter().take(child_level + 1) {
      index_offset += childs_in_level.len();
    }

    for block in &mut parsing_data.blocks[child_level] {
      let settings = block.get_settings_mut();
      for child in &mut settings.children {
        *child += index_offset;
      }

      ordered_blocks.push(block.clone());
    }
  }

  let mut section_map = HashMap::default();

  for i in 0..level_0_size {
    if let Block::Section { id: name, settings } = &ordered_blocks[i] {
      add_section(
        name,
        &settings.children,
        None,
        i,
        &mut section_map,
        &ordered_blocks,
      );
    }
  }

  validate_diverts(&ordered_blocks, &section_map)?;
  check_duplicate_sections(&ordered_blocks, level_0_size)?;

  Ok(Database {
    blocks: ordered_blocks,
    sections: section_map,
    i18n: parsing_data.i18n,
    config: parsing_data.config,
  })
}

pub fn parse_database_str(input: &str, config: &Config) -> Result<Database, PalabritasError> {
  let token = parse_str(input, Rule::Database)?;
  let parsing_data = ParsingData::new(config.clone(), String::default());
  parse_database(token, parsing_data)
}

pub fn parse_text_str(input: &str) -> Result<Block, PalabritasError> {
  let token = parse_str(input, Rule::Text)?;
  parse_text(token, &String::default())
}

pub fn parse_named_bucket_str(input: &str) -> Result<Block, PalabritasError> {
  let token = parse_str(input, Rule::NamedBucket)?;
  parse_named_bucket(token, &String::default())
}

pub fn parse_chance_str(input: &str) -> Result<Chance, PalabritasError> {
  let token = parse_str(input, Rule::Chance)?;
  parse_chance(token, &String::default())
}

pub fn parse_condition_str(input: &str, config: &Config) -> Result<Condition, PalabritasError> {
  let token = parse_str(input, Rule::Condition)?;
  parse_condition(token, config, &String::default())
}

pub fn parse_choice_str(input: &str) -> Result<Block, PalabritasError> {
  let token = parse_str(input, Rule::Choice)?;
  parse_choice(token, &String::default())
}

pub fn parse_section_str(input: &str, config: &Config) -> Result<Block, PalabritasError> {
  let token = parse_str(input, Rule::Section)?;
  let mut parsing_data = ParsingData::new(config.clone(), String::default());
  parse_section(token, &mut parsing_data, 0)
}

pub fn parse_tag_str(input: &str) -> Result<String, PalabritasError> {
  let token = parse_str(input, Rule::Tag)?;
  parse_tag(token, &String::default())
}

pub fn parse_function_str(input: &str) -> Result<Function, PalabritasError> {
  let token = parse_str(input, Rule::Function)?;
  parse_function(token, &String::default())
}

pub fn parse_divert_str(input: &str) -> Result<Block, PalabritasError> {
  let token = parse_str(input, Rule::Divert)?;
  parse_divert(token, &String::default())
}

pub fn parse_boomerang_divert_str(input: &str) -> Result<Block, PalabritasError> {
  let token = parse_str(input, Rule::BoomerangDivert)?;
  parse_boomerang_divert(token, &String::default())
}

pub fn parse_modifier_str(input: &str, config: &Config) -> Result<Modifier, PalabritasError> {
  let token = parse_str(input, Rule::Modifier)?;
  parse_modifier(token, config, &String::default())
}

pub fn parse_frequency_str(
  input: &str,
  config: &Config,
) -> Result<FrequencyModifier, PalabritasError> {
  let token = parse_str(input, Rule::Frequency)?;
  parse_frequency(token, config, &String::default())
}

pub fn parse_requirement_str(input: &str, config: &Config) -> Result<Requirement, PalabritasError> {
  let token = parse_str(input, Rule::Requirement)?;
  parse_requirement(token, config, &String::default())
}

pub fn parse_comparison_operator_str(input: &str) -> Result<ComparisonOperator, PalabritasError> {
  let token = parse_str(input, Rule::ComparisonOperator)?;
  parse_comparison_operator(token, &String::default())
}

pub fn parse_modifier_operator_str(input: &str) -> Result<ModifierOperator, PalabritasError> {
  let token = parse_str(input, Rule::ModifierOperator)?;
  parse_modifier_operator(token, &String::default())
}

fn parse_str(input: &str, rule: Rule) -> Result<Pair<'_, Rule>, PalabritasError> {
  match PalabritasParser::parse(rule, input) {
    Ok(mut pairs) => match pairs.next() {
      Some(token) => Ok(token),
      None => Err(PalabritasError::ParseError {
        reason: format!("{:?} not found", rule),
        script: Script {
          line: 1,
          col: 1,
          file: String::default(),
        },
      }),
    },
    Err(error) => {
      let (line, col) = match error.line_col {
        LineColLocation::Pos(line_col) => line_col,
        LineColLocation::Span(start, _) => (start.0, start.1),
      };

      Err(PalabritasError::ParseError {
        reason: error.to_string(),
        script: Script {
          line,
          col,
          file: String::default(),
        },
      })
    }
  }
}

fn parse_section(
  token: Pair<Rule>,
  parsing_data: &mut ParsingData,
  child_order: usize,
) -> Result<Block, PalabritasError> {
  match_rule(&token, Rule::Section, &parsing_data.file)?;
  if parsing_data.blocks.is_empty() {
    parsing_data.blocks.push(Vec::default());
  }

  parsing_data.blocks[child_order].push(Block::default());
  let block_id = parsing_data.blocks[child_order].len() - 1;
  let script = Script {
    line: token.line_col().0,
    col: token.line_col().1,
    file: parsing_data.file.clone(),
  };

  let string = token.as_str().to_string();

  let mut settings = BlockSettings {
    script: Script {
      file: parsing_data.file.clone(),
      line: token.line_col().0,
      col: token.line_col().1,
    },
    ..Default::default()
  };
  let mut id: String = String::default();
  //Section = {"#" ~ " "* ~ Identifier ~ " "* ~ Command* ~ NewLine ~ ( NewLine | NewBlock | Subsection )* }
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Identifier => {
        id = inner_token.as_str().to_string();
        parsing_data.current_section = Some(Section {
          name: id.clone(),
          parent: None,
        });
        settings.section = parsing_data.current_section.clone();
      }
      Rule::Chance => {
        settings.chance = parse_chance(inner_token, &parsing_data.file)?;
        if let Chance::Frequency(_) = settings.chance {
          return Err(PalabritasError::FrequencyOutOfBucket(script, string));
        }
      }
      Rule::Command => {
        add_command_to_settings(
          inner_token,
          &mut settings,
          &parsing_data.config,
          &parsing_data.file,
        )?;
      }
      Rule::NewBlock => {
        for inner_blocks_token in get_blocks_from_new_block(inner_token) {
          parse_block(inner_blocks_token, parsing_data, child_order + 1)?;
          settings
            .children
            .push(parsing_data.blocks[child_order + 1].len() - 1);
        }
      }
      Rule::Subsection => {
        parse_subsection(inner_token, parsing_data, child_order + 1)?;
        parsing_data.current_section = settings.section.clone();
        settings
          .children
          .push(parsing_data.blocks[child_order + 1].len() - 1);
      }
      _ => {}
    }
  }

  let section = Block::Section { id, settings };
  check_invalid_frequency(&section, script, string, &parsing_data.blocks, child_order)?;
  parsing_data.blocks[0][block_id] = section.clone();

  Ok(section)
}

fn parse_subsection(
  token: Pair<Rule>,
  parsing_data: &mut ParsingData,
  child_order: usize,
) -> Result<(), PalabritasError> {
  match token.as_rule() {
    Rule::Subsection => {}
    Rule::Subsubsection => {}
    Rule::Subsubsubsection => {}
    _ => {
      return Err(PalabritasError::UnexpectedRule {
        script: Script {
          line: token.line_col().0,
          col: token.line_col().1,
          file: parsing_data.file.clone(),
        },
        expected_rule: Rule::Subsection,
        rule_found: token.as_rule(),
      });
    }
  };

  while child_order >= parsing_data.blocks.len() {
    parsing_data.blocks.push(Vec::default());
  }

  parsing_data.blocks[child_order].push(Block::default());
  let block_id = parsing_data.blocks[child_order].len() - 1;
  let script = Script {
    line: token.line_col().0,
    col: token.line_col().1,
    file: parsing_data.file.clone(),
  };

  let string = token.as_str().to_string();

  let mut settings = BlockSettings {
    script: Script {
      file: parsing_data.file.clone(),
      line: token.line_col().0,
      col: token.line_col().1,
    },
    ..Default::default()
  };

  let mut id: String = String::default();

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Identifier => {
        id = inner_token.as_str().to_string();
        if let Some(section) = &parsing_data.current_section {
          parsing_data.current_section = Some(Section {
            name: id.clone(),
            parent: Some(Box::new(section.clone())),
          });
        }
        settings.section = parsing_data.current_section.clone();
      }
      Rule::Command => {
        add_command_to_settings(
          inner_token,
          &mut settings,
          &parsing_data.config,
          &parsing_data.file,
        )?;
      }
      Rule::Chance => {
        settings.chance = parse_chance(inner_token, &parsing_data.file)?;
        if let Chance::Frequency(_) = settings.chance {
          return Err(PalabritasError::FrequencyOutOfBucket(script, string));
        }
      }
      Rule::NewBlock => {
        for inner_blocks_token in get_blocks_from_new_block(inner_token) {
          parse_block(inner_blocks_token, parsing_data, child_order + 1)?;
          settings
            .children
            .push(parsing_data.blocks[child_order + 1].len() - 1);
        }
      }
      Rule::Subsubsection => {
        parse_subsection(inner_token, parsing_data, child_order + 1)?;
        settings
          .children
          .push(parsing_data.blocks[child_order + 1].len() - 1);
      }
      Rule::Subsubsubsection => {
        parse_subsection(inner_token, parsing_data, child_order + 1)?;
        settings
          .children
          .push(parsing_data.blocks[child_order + 1].len() - 1);
      }
      _ => {}
    }
  }

  let subsection = Block::Section { id, settings };
  check_invalid_frequency(
    &subsection,
    script,
    string,
    &parsing_data.blocks,
    child_order,
  )?;
  parsing_data.blocks[child_order][block_id] = subsection;

  Ok(())
}
fn parse_block(
  token: Pair<Rule>,
  parsing_data: &mut ParsingData,
  child_order: usize,
) -> Result<(), PalabritasError> {
  match_rule(&token, Rule::Block, &parsing_data.file)?;

  //    (NamedBucket | Choice | Text | Divery)  ~  " "* ~ Command* ~ " "* ~ (NEWLINE | EOI) ~ NewBlock*
  let mut block = Block::default();
  let line_col = token.line_col();
  let script = Script {
    line: token.line_col().0,
    col: token.line_col().1,
    file: parsing_data.file.clone(),
  };

  let string = token.as_str().to_string();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Text => {
        block = parse_text(inner_token, &parsing_data.file)?;
      }
      Rule::NamedBucket => {
        block = parse_named_bucket(inner_token, &parsing_data.file)?;
      }
      Rule::Choice => {
        block = parse_choice(inner_token, &parsing_data.file)?;
      }
      Rule::Divert => {
        block = parse_divert(inner_token, &parsing_data.file)?;
      }
      Rule::BoomerangDivert => {
        block = parse_boomerang_divert(inner_token, &parsing_data.file)?;
      }
      Rule::Command => {
        add_command_to_block(
          inner_token,
          &mut block,
          &parsing_data.config,
          &parsing_data.file,
        )?;
      }
      Rule::NewBlock => {
        for inner_blocks_token in get_blocks_from_new_block(inner_token) {
          let settings = block.get_settings_mut();
          parse_block(inner_blocks_token, parsing_data, child_order + 1)?;
          settings
            .children
            .push(parsing_data.blocks[child_order + 1].len() - 1);
        }
      }
      _ => {}
    }
  }

  block.get_settings_mut().section = parsing_data.current_section.clone();

  while child_order >= parsing_data.blocks.len() {
    parsing_data.blocks.push(Vec::default());
  }

  update_i18n(&mut block, &mut parsing_data.i18n, line_col.0);

  block.get_settings_mut().script = Script {
    file: parsing_data.file.clone(),
    line: line_col.0,
    col: line_col.1,
  };

  parsing_data.blocks[child_order].push(block);

  let block_id = parsing_data.blocks[child_order].len() - 1;

  if let Block::Bucket {
    name: _,
    settings: _,
  } = parsing_data.blocks[child_order][block_id]
  {
    validate_bucket_data(
      block_id,
      &mut parsing_data.blocks,
      child_order,
      line_col,
      &parsing_data.file,
    )?;

    update_children_probabilities_to_frequency(
      parsing_data.blocks[child_order].len() - 1,
      &mut parsing_data.blocks,
      child_order,
    );
  } else if is_child_unnamed_bucket(block_id, &parsing_data.blocks, child_order) {
    make_childs_bucket(block_id, &mut parsing_data.blocks, child_order);
  }

  check_invalid_frequency(
    &parsing_data.blocks[child_order][block_id],
    script,
    string,
    &parsing_data.blocks,
    child_order,
  )?;

  Ok(())
}

fn check_invalid_frequency(
  block: &Block,
  script: Script,
  string: String,
  blocks: &[Vec<Block>],
  child_order: usize,
) -> Result<(), PalabritasError> {
  fn freq_modifier_matches_chance(
    settings: &BlockSettings,
    script: &Script,
    string: &str,
  ) -> Result<(), PalabritasError> {
    if settings.frequency_modifiers.is_empty() {
      Ok(())
    } else {
      match settings.chance {
        Chance::Frequency(_) => Ok(()),
        _ => Err(PalabritasError::FrequencyModifierWithoutFrequencyChance(
          script.clone(),
          string.to_string(),
        )),
      }
    }
  }

  match block {
    Block::Bucket {
      name: _,
      settings: _,
    } => freq_modifier_matches_chance(block.get_settings(), &script, &string),

    Block::Section { id: _, settings: _ } => {
      if let Chance::Frequency(_) = block.get_settings().chance {
        Err(PalabritasError::FrequencyOutOfBucket(script, string))
      } else {
        Ok(())
      }
    }
    _ => {
      freq_modifier_matches_chance(block.get_settings(), &script, &string)?;
      if child_order == 0 {
        if let Chance::Frequency(_) = block.get_settings().chance {
          return Err(PalabritasError::FrequencyOutOfBucket(script, string));
        }
      }
      for child in &block.get_settings().children {
        if let Chance::Frequency(_) = blocks[child_order + 1][*child].get_settings().chance {
          return Err(PalabritasError::FrequencyOutOfBucket(script, string));
        }
      }
      Ok(())
    }
  }
}
fn update_i18n(block: &mut Block, i18n: &mut I18n, line: usize) {
  if let Some(i18n_id) = block.get_i18n_id() {
    if let Some(db) = i18n.strings.get_mut(&i18n.default_locale) {
      let new_id = line.to_string();
      db.insert(new_id.clone(), i18n_id);

      match block {
        Block::Text {
          ref mut id,
          settings: _,
        } => {
          *id = new_id;
        }
        Block::Choice {
          ref mut id,
          settings: _,
        } => {
          *id = new_id;
        }
        _ => {}
      }
    }
  }
}

fn is_child_unnamed_bucket(block: usize, blocks: &Vec<Vec<Block>>, child_order: usize) -> bool {
  let block = &blocks[child_order][block];
  let children = &block.get_settings().children;

  if children.len() < 2 || child_order + 1 >= blocks.len() {
    return false;
  }

  let mut total_probability = 0.;
  let mut is_frequency: bool = false;
  for i in 0..blocks[child_order + 1].len() {
    for child in children {
      if *child == i {
        match blocks[child_order + 1][i].get_settings().chance {
          cuentitos_common::Chance::None => {
            return false;
          }
          cuentitos_common::Chance::Frequency(_) => {
            is_frequency = true;
          }
          cuentitos_common::Chance::Probability(value) => {
            total_probability += value;
          }
        }
      }
    }
  }

  if is_frequency && total_probability > 0. {
    return false;
  }

  if is_frequency {
    return true;
  }

  total_probability == 1.
}

fn make_childs_bucket(block_id: usize, blocks: &mut Vec<Vec<Block>>, child_order: usize) {
  if child_order + 1 >= blocks.len() {
    return;
  }

  update_children_probabilities_to_frequency(block_id, blocks, child_order);

  let block = blocks[child_order][block_id].clone();
  let block_settings = block.get_settings();
  let bucket = Block::Bucket {
    name: None,
    settings: BlockSettings {
      children: block_settings.children.clone(),
      ..Default::default()
    },
  };

  blocks[child_order].push(bucket);

  move_to_lower_level(blocks[child_order].len() - 1, blocks, child_order);

  let new_children = vec![blocks[child_order + 1].len() - 1];
  let block = &mut blocks[child_order][block_id];
  let block_settings = block.get_settings_mut();
  block_settings.children = new_children;
}

fn validate_bucket_data(
  bucket: usize,
  blocks: &mut [Vec<Block>],
  child_order: usize,
  line_col: (usize, usize),
  file: &str,
) -> Result<(), PalabritasError> {
  let bucket = &blocks[child_order][bucket];
  let settings = bucket.get_settings();

  let bucket_name = match bucket {
    Block::Bucket {
      name: Some(string),
      settings: _,
    } => string.clone(),
    _ => String::default(),
  };
  let mut frequency_found = false;
  let mut chance_found = false;
  let mut total_probability = 0.;

  let mut inner_line = line_col.0;
  for child in &settings.children {
    inner_line += 1;
    let child_block = &blocks[child_order + 1][*child];
    let child_settings = child_block.get_settings();
    match child_settings.chance {
      Chance::None => {
        let string = match child_block {
          Block::Bucket {
            name: Some(name),
            settings: _,
          } => name.clone(),
          Block::Text { id, settings: _ } => id.clone(),
          Block::Choice { id, settings: _ } => id.clone(),
          _ => String::default(),
        };
        return Err(PalabritasError::BucketMissingProbability(
          Script {
            line: inner_line,
            col: 1,
            file: file.to_string(),
          },
          string,
        ));
      }
      Chance::Frequency(_) => frequency_found = true,
      Chance::Probability(probability) => {
        chance_found = true;
        total_probability += probability;
      }
    }

    if frequency_found && chance_found {
      return Err(PalabritasError::BucketHasFrequenciesAndChances(
        Script {
          line: line_col.0,
          col: line_col.1,
          file: file.to_string(),
        },
        bucket_name,
      ));
    }
  }

  if chance_found && total_probability != 1. {
    return Err(PalabritasError::BucketSumIsNot1(
      Script {
        line: line_col.0,
        col: line_col.1,
        file: file.to_string(),
      },
      bucket_name,
    ));
  }

  Ok(())
}

fn move_to_lower_level(index: usize, blocks: &mut Vec<Vec<Block>>, child_order: usize) {
  update_higher_level(index, blocks, child_order);

  let child_count = blocks[child_order][index].get_settings().children.len();
  for i in 0..child_count {
    for e in i..child_count {
      if blocks[child_order][index].get_settings().children[e]
        > blocks[child_order][index].get_settings().children[i]
      {
        blocks[child_order][index].get_settings_mut().children[e] -= 1;
      }
    }

    let child_index = blocks[child_order][index].get_settings().children[i];
    move_to_lower_level(child_index, blocks, child_order + 1);
  }

  let mut block: Block = blocks[child_order].remove(index);
  if blocks.len() <= child_order + 1 {
    blocks.push(Vec::default());
  }

  let mut new_children = Vec::default();
  for i in 0..child_count {
    let new_child_index = blocks[child_order + 2].len() - 1 - i;
    new_children.push(new_child_index);
  }

  new_children.reverse();

  block.get_settings_mut().children = new_children;
  blocks[child_order + 1].push(block);

  fn update_higher_level(index: usize, blocks: &mut [Vec<Block>], child_order: usize) {
    if child_order == 0 {
      return;
    }
    for higher_level_block in &mut blocks[child_order - 1] {
      let higher_level_settings = higher_level_block.get_settings_mut();
      if higher_level_settings.children.contains(&index) {
        continue;
      }
      for i in 0..higher_level_settings.children.len() {
        if higher_level_settings.children[i] > index {
          higher_level_settings.children[i] -= 1;
        }
      }
    }
  }
}

fn update_children_probabilities_to_frequency(
  block: usize,
  blocks: &mut Vec<Vec<Block>>,
  child_order: usize,
) {
  if child_order + 1 >= blocks.len() {
    return;
  }
  let block = blocks[child_order][block].clone();
  let children = &block.get_settings().children;

  for child in children.iter().rev() {
    let child = &mut blocks[child_order + 1][*child];
    let child_settings = child.get_settings_mut();
    if let Chance::Probability(chance) = child_settings.chance {
      child_settings.chance = Chance::Frequency((chance * 100.) as u32);
    }
  }
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

fn parse_named_bucket(token: Pair<Rule>, file: &str) -> Result<Block, PalabritasError> {
  match_rule(&token, Rule::NamedBucket, file)?;

  let mut name = None;
  let mut settings = BlockSettings::default();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Chance => {
        settings.chance = parse_chance(inner_token, file)?;
      }
      Rule::SnakeCase => {
        name = Some(inner_token.as_str().to_string());
      }
      _ => {}
    }
  }

  Ok(Block::Bucket { name, settings })
}

fn parse_choice(token: Pair<Rule>, file: &str) -> Result<Block, PalabritasError> {
  match_rule(&token, Rule::Choice, file)?;

  let mut text = String::default();
  let mut settings = BlockSettings::default();

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Chance => {
        settings.chance = parse_chance(inner_token, file)?;
      }
      Rule::String => {
        text = inner_token.as_str().to_string();
      }
      _ => {}
    }
  }

  Ok(Block::Choice { id: text, settings })
}

fn parse_text(token: Pair<Rule>, file: &str) -> Result<Block, PalabritasError> {
  match_rule(&token, Rule::Text, file)?;

  let mut text = String::default();
  let mut settings = BlockSettings::default();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Chance => {
        settings.chance = parse_chance(inner_token, file)?;
      }
      Rule::String => {
        text = inner_token.as_str().to_string();
      }
      _ => {}
    }
  }

  Ok(Block::Text { id: text, settings })
}

fn add_command_to_settings(
  token: Pair<Rule>,
  settings: &mut BlockSettings,
  config: &Config,
  file: &str,
) -> Result<(), PalabritasError> {
  match_rule(&token, Rule::Command, file)?;

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      //Command = {NewLine ~ (Indentation | " ")* ~ (Requirement | Frequency | Modifier | Function | Unique | Tag) }
      Rule::Requirement => {
        let requirement = parse_requirement(inner_token, config, file)?;
        settings.requirements.push(requirement);
      }
      Rule::Frequency => {
        let frequency = parse_frequency(inner_token, config, file)?;
        settings.frequency_modifiers.push(frequency);
      }
      Rule::Modifier => {
        let modifier = parse_modifier(inner_token, config, file)?;
        settings.modifiers.push(modifier);
      }
      Rule::Unique => {
        settings.unique = true;
      }
      Rule::Tag => {
        let tag = parse_tag(inner_token, file)?;
        settings.tags.push(tag);
      }
      Rule::Function => {
        let function = parse_function(inner_token, file)?;
        settings.functions.push(function);
      }
      _ => {}
    }
  }

  Ok(())
}
fn add_command_to_block(
  token: Pair<Rule>,
  block: &mut Block,
  config: &Config,
  file: &str,
) -> Result<(), PalabritasError> {
  let settings = block.get_settings_mut();
  add_command_to_settings(token, settings, config, file)
}

fn parse_function(token: Pair<Rule>, file: &str) -> Result<Function, PalabritasError> {
  match_rule(&token, Rule::Function, file)?;

  let mut name = String::default();
  let mut parameters = Vec::default();
  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Identifier {
      name = inner_token.as_str().to_string();
    }
    if inner_token.as_rule() == Rule::Value {
      parameters.push(inner_token.as_str().to_string());
    }
  }
  Ok(Function { name, parameters })
}

fn parse_tag(token: Pair<Rule>, file: &str) -> Result<String, PalabritasError> {
  match_rule(&token, Rule::Tag, file)?;

  let mut name = String::default();
  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Identifier {
      name = inner_token.as_str().to_string();
    }
  }
  Ok(name)
}
fn parse_divert(token: Pair<Rule>, file: &str) -> Result<Block, PalabritasError> {
  match_rule(&token, Rule::Divert, file)?;
  //Divert = { Chance? ~ "->"  ~ " "* ~ Identifier ~ ("/" ~ Identifier)* }

  let mut section_names = Vec::default();

  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Identifier {
      section_names.push(inner_token.as_str().to_string());
    }
  }

  let next = match section_names == vec!["END"] {
    true => NextBlock::EndOfFile,
    false => {
      let mut section = None;
      for name in section_names {
        section = Some(Box::new(Section {
          name,
          parent: section,
        }));
      }
      NextBlock::Section(*section.unwrap())
    }
  };

  Ok(Block::Divert {
    next,
    settings: BlockSettings::default(),
  })
}

fn parse_boomerang_divert(token: Pair<Rule>, file: &str) -> Result<Block, PalabritasError> {
  match_rule(&token, Rule::BoomerangDivert, file)?;

  let mut section_names = Vec::default();

  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Identifier {
      section_names.push(inner_token.as_str().to_string());
    }
  }

  let next = match section_names == vec!["END"] {
    true => NextBlock::EndOfFile,
    false => {
      let mut section = None;
      for name in section_names {
        section = Some(Box::new(Section {
          name,
          parent: section,
        }));
      }
      NextBlock::Section(*section.unwrap())
    }
  };

  Ok(Block::BoomerangDivert {
    next,
    settings: BlockSettings::default(),
  })
}

fn parse_modifier(
  token: Pair<Rule>,
  config: &Config,
  file: &str,
) -> Result<Modifier, PalabritasError> {
  match_rule(&token, Rule::Modifier, file)?;
  //Modifier = { "set" ~ " "+ ~ ( (Identifier ~ " "+ ~ ModifierOperator? ~ Value) | (NotOperator? ~ " "* ~ Identifier) ) ~ " "* }
  let script = Script {
    line: token.line_col().0,
    col: token.line_col().1,
    file: file.to_string(),
  };
  let string = token.as_str().to_string();
  let mut modifier = Modifier::default();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Identifier => {
        modifier.variable = inner_token.as_str().to_string();
      }

      Rule::Value => {
        modifier.value = inner_token.as_str().to_string();
      }

      Rule::ModifierOperator => {
        modifier.operator = parse_modifier_operator(inner_token, file)?;
      }

      Rule::NotOperator => {
        modifier.value = false.to_string();
      }

      _ => {}
    }
  }

  check_variable_existance(&modifier.variable, config, &script, &string)?;
  check_variable_value_matches_type(
    &modifier.variable,
    &modifier.value,
    config,
    &script,
    &string,
  )?;
  check_variable_modifier_operator_matches_type(
    &modifier.variable,
    &modifier.operator,
    config,
    &script,
    &string,
  )?;

  if modifier.operator == ModifierOperator::Divide && modifier.value == *"0" {
    Err(PalabritasError::DivisionByZero(script, string))
  } else {
    Ok(modifier)
  }
}

fn check_variable_existance(
  variable: &str,
  config: &Config,
  script: &Script,
  string: &str,
) -> Result<(), PalabritasError> {
  if config.variables.get(variable).is_some() {
    Ok(())
  } else {
    Err(PalabritasError::VariableDoesntExist {
      script: script.clone(),
      string: string.to_string(),
      variable: variable.to_string(),
    })
  }
}

fn check_duplicate_sections(
  blocks: &Vec<Block>,
  level_0_size: usize,
) -> Result<(), PalabritasError> {
  fn check_related_duplicates(
    settings: &BlockSettings,
    blocks: &[Block],
  ) -> Result<(), PalabritasError> {
    let mut names_taken: HashMap<String, Script> = HashMap::default();

    for child in &settings.children {
      if let Block::Section { id, settings } = &blocks[*child] {
        names_taken.insert(id.clone(), settings.script.clone());
      }
    }

    for child in &settings.children {
      let grand_children = if let Block::Section {
        id: _,
        settings: child_settings,
      } = &blocks[*child]
      {
        &child_settings.children
      } else {
        continue;
      };

      for grand_child in grand_children {
        let grand_children = if let Block::Section {
          id,
          settings: grand_child_settings,
        } = &blocks[*grand_child]
        {
          (id, grand_child_settings)
        } else {
          continue;
        };

        if let Some(script) = names_taken.get(grand_children.0) {
          return Err(PalabritasError::SubsectioNamedAfterUpperSection {
            subsection_script: Box::new(grand_children.1.script.clone()),
            section_script: Box::new(script.clone()),
            subsection: grand_children.0.clone(),
          });
        }
      }
    }

    Ok(())
  }
  fn check_if_subsection_exists(
    id: &SectionName,
    script: &Script,
    sections: &HashMap<Section, Script>,
  ) -> Result<(), PalabritasError> {
    for (other_section, other_script) in sections {
      if other_section.parent.is_some() && other_section.name == *id {
        return Err(PalabritasError::SubsectioNamedAfterUpperSection {
          subsection_script: Box::new(other_script.clone()),
          section_script: Box::new(script.clone()),
          subsection: id.clone(),
        });
      }
    }

    Ok(())
  }

  fn check_duplicate(
    settings: &BlockSettings,
    sections: &mut HashMap<Section, Script>,
  ) -> Result<(), PalabritasError> {
    if let Some(section) = &settings.section {
      if let Some(first_appearance) = sections.get(section) {
        let first_appearance = Box::new(first_appearance.clone());
        let second_appearance = Box::new(settings.script.clone());
        return Err(PalabritasError::DuplicatedSection {
          first_appearance,
          section: section.clone(),
          second_appearance,
        });
      }
      sections.insert(section.clone(), settings.script.clone());
    }
    Ok(())
  }

  let mut sections: HashMap<Section, Script> = HashMap::default();

  for block in blocks {
    if let Block::Section { id: _, settings } = block {
      check_duplicate(settings, &mut sections)?;
    }
  }

  for i in 0..level_0_size {
    if let Block::Section { id, settings } = &blocks[i] {
      check_if_subsection_exists(id, &settings.script, &sections)?;
      check_related_duplicates(settings, blocks)?;
    }
  }

  Ok(())
}
fn validate_diverts(
  blocks: &Vec<Block>,
  section_map: &HashMap<Section, BlockId>,
) -> Result<(), PalabritasError> {
  fn append_to_parent(value: Section, into: &mut Section) {
    match &mut into.parent {
      Some(parent) => append_to_parent(value, parent),
      None => {
        into.parent = Some(Box::new(value));
      }
    }
  }

  for block in blocks {
    if let Block::Divert {
      next: NextBlock::Section(section),
      settings: _,
    } = block
    {
      if section_map.contains_key(section) {
        continue;
      }

      if let Some(current_section) = block.get_settings().section.clone() {
        let mut section = section.clone();
        append_to_parent(current_section, &mut section);
        if section_map.contains_key(&section) {
          continue;
        }

        section.parent = section.parent.unwrap().parent;

        if section_map.contains_key(&section) {
          continue;
        }
      }

      return Err(PalabritasError::SectionDoesntExist {
        script: block.get_settings().script.clone(),
        section: section.clone(),
      });
    }
  }

  Ok(())
}

fn check_variable_value_matches_type(
  variable: &str,
  value: &str,
  config: &Config,
  script: &Script,
  string: &str,
) -> Result<(), PalabritasError> {
  if let Some(kind) = config.variables.get(variable) {
    match kind {
      cuentitos_common::VariableKind::Integer => {
        if value.parse::<i32>().is_ok() {
          Ok(())
        } else {
          Err(PalabritasError::InvalidVariableValue {
            script: Box::new(script.clone()),
            variable: variable.to_string(),
            value: value.to_string(),
            variable_type: kind.clone(),
            string: string.to_string(),
          })
        }
      }
      cuentitos_common::VariableKind::Float => {
        if value.parse::<f32>().is_ok() {
          Ok(())
        } else {
          Err(PalabritasError::InvalidVariableValue {
            script: Box::new(script.clone()),
            variable: variable.to_string(),
            value: value.to_string(),
            variable_type: kind.clone(),
            string: string.to_string(),
          })
        }
      }
      cuentitos_common::VariableKind::Bool => {
        if value.parse::<bool>().is_ok() {
          Ok(())
        } else {
          Err(PalabritasError::InvalidVariableValue {
            script: Box::new(script.clone()),
            variable: variable.to_string(),
            value: value.to_string(),
            variable_type: kind.clone(),
            string: string.to_string(),
          })
        }
      }
      cuentitos_common::VariableKind::String => Ok(()),
      cuentitos_common::VariableKind::Enum(variants) => {
        match variants.contains(&value.to_string()) {
          true => Ok(()),
          false => Err(PalabritasError::InvalidVariableValue {
            script: Box::new(script.clone()),
            variable: variable.to_string(),
            value: value.to_string(),
            variable_type: kind.clone(),
            string: string.to_string(),
          }),
        }
      }
    }
  } else {
    Err(PalabritasError::VariableDoesntExist {
      script: script.clone(),
      variable: variable.to_string(),
      string: string.to_string(),
    })
  }
}

fn check_variable_modifier_operator_matches_type(
  variable: &str,
  operator: &ModifierOperator,
  config: &Config,
  script: &Script,
  string: &str,
) -> Result<(), PalabritasError> {
  if let Some(kind) = config.variables.get(variable) {
    match kind {
      cuentitos_common::VariableKind::Integer => Ok(()),
      cuentitos_common::VariableKind::Float => Ok(()),
      _ => match operator {
        ModifierOperator::Set => Ok(()),
        _ => Err(PalabritasError::InvalidVariableOperator {
          script: Box::new(script.clone()),
          variable: variable.to_string(),
          operator: format!("{}", operator),
          variable_type: kind.clone(),
          string: string.to_string(),
        }),
      },
    }
  } else {
    Err(PalabritasError::VariableDoesntExist {
      script: script.clone(),
      variable: variable.to_string(),
      string: string.to_string(),
    })
  }
}

fn check_variable_comparison_operator_matches_type(
  variable: &str,
  operator: &ComparisonOperator,
  config: &Config,
  script: &Script,
  string: &str,
) -> Result<(), PalabritasError> {
  if let Some(kind) = config.variables.get(variable) {
    match kind {
      cuentitos_common::VariableKind::Integer => Ok(()),
      cuentitos_common::VariableKind::Float => Ok(()),
      _ => match operator {
        ComparisonOperator::Equal => Ok(()),
        ComparisonOperator::NotEqual => Ok(()),
        _ => Err(PalabritasError::InvalidVariableOperator {
          script: Box::new(script.clone()),
          variable: variable.to_string(),
          operator: format!("{}", operator),
          variable_type: kind.clone(),
          string: string.to_string(),
        }),
      },
    }
  } else {
    Err(PalabritasError::VariableDoesntExist {
      script: script.clone(),
      variable: variable.to_string(),
      string: string.to_string(),
    })
  }
}

fn parse_modifier_operator(
  token: Pair<Rule>,
  file: &str,
) -> Result<ModifierOperator, PalabritasError> {
  //ModifierOperator = {"+" | "-" | "*" | "/" | "="}
  match_rule(&token, Rule::ModifierOperator, file)?;

  match token.as_str() {
    "+" => Ok(ModifierOperator::Add),
    "-" => Ok(ModifierOperator::Substract),
    "*" => Ok(ModifierOperator::Multiply),
    "/" => Ok(ModifierOperator::Divide),
    "=" => Ok(ModifierOperator::Set),
    _ => Ok(ModifierOperator::default()),
  }
}
fn parse_frequency(
  token: Pair<Rule>,
  config: &Config,
  file: &str,
) -> Result<FrequencyModifier, PalabritasError> {
  match_rule(&token, Rule::Frequency, file)?;

  let mut frequency = FrequencyModifier::default();
  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Condition => {
        frequency.condition = parse_condition(inner_token, config, file)?;
      }

      Rule::Float | Rule::Integer => {
        let value = inner_token.as_str().parse::<i32>().unwrap();
        frequency.value = value;
      }
      _ => {}
    }
  }

  Ok(frequency)
}

fn parse_requirement(
  token: Pair<Rule>,
  config: &Config,
  file: &str,
) -> Result<Requirement, PalabritasError> {
  match_rule(&token, Rule::Requirement, file)?;

  let mut condition = Condition::default();
  for inner_token in token.into_inner() {
    if inner_token.as_rule() == Rule::Condition {
      condition = parse_condition(inner_token, config, file)?;
    }
  }

  Ok(Requirement { condition })
}

fn parse_condition(
  token: Pair<Rule>,
  config: &Config,
  file: &str,
) -> Result<Condition, PalabritasError> {
  match_rule(&token, Rule::Condition, file)?;
  //Condition = { ( Identifier ~ " "* ~ ( ComparisonOperator ~ " "* )? ~ Value? ) | ( NotEqualOperator? ~ " "* ~ Identifier ~ " "*) }
  let script = Script {
    line: token.line_col().0,
    col: token.line_col().1,
    file: file.to_string(),
  };

  let string = token.as_str().to_string();
  let mut condition = Condition::default();

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Identifier => {
        condition.variable = inner_token.as_str().to_string();
      }
      Rule::ComparisonOperator => {
        condition.operator = parse_comparison_operator(inner_token, file)?;
      }
      Rule::NotOperator => {
        condition.operator = ComparisonOperator::NotEqual;
      }
      Rule::Value => {
        condition.value = inner_token.as_str().to_string();
      }
      _ => {}
    }
  }

  check_variable_existance(&condition.variable, config, &script, &string)?;
  check_variable_value_matches_type(
    &condition.variable,
    &condition.value,
    config,
    &script,
    &string,
  )?;
  check_variable_comparison_operator_matches_type(
    &condition.variable,
    &condition.operator,
    config,
    &script,
    &string,
  )?;
  Ok(condition)
}

fn parse_comparison_operator(
  token: Pair<Rule>,
  file: &str,
) -> Result<ComparisonOperator, PalabritasError> {
  match_rule(&token, Rule::ComparisonOperator, file)?;

  match token.as_str() {
    "!=" => Ok(ComparisonOperator::NotEqual),
    "!" => Ok(ComparisonOperator::NotEqual),
    "=" => Ok(ComparisonOperator::Equal),
    "<=" => Ok(ComparisonOperator::LessOrEqualThan),
    ">=" => Ok(ComparisonOperator::GreaterOrEqualThan),
    "<" => Ok(ComparisonOperator::LessThan),
    ">" => Ok(ComparisonOperator::GreaterThan),
    _ => Ok(ComparisonOperator::default()),
  }
}

fn parse_chance(token: Pair<Rule>, file: &str) -> Result<Chance, PalabritasError> {
  match_rule(&token, Rule::Chance, file)?;

  for inner_token in token.into_inner() {
    match inner_token.as_rule() {
      Rule::Float => {
        let value = inner_token.as_str().parse::<f32>().unwrap();
        return Ok(Chance::Probability(value));
      }
      Rule::Percentage => {
        if let Some(integer) = inner_token.into_inner().next() {
          let value = integer.as_str().parse::<u64>().unwrap();
          return Ok(Chance::Probability(value as f32 / 100.));
        }
      }
      Rule::Integer => {
        let value = inner_token.as_str().parse::<u32>().unwrap();
        return Ok(Chance::Frequency(value));
      }
      _ => {}
    }
  }

  Ok(Chance::None)
}

fn match_rule(token: &Pair<Rule>, expected_rule: Rule, file: &str) -> Result<(), PalabritasError> {
  if token.as_rule() != expected_rule {
    return Err(PalabritasError::UnexpectedRule {
      script: Script {
        line: token.line_col().0,
        col: token.line_col().1,
        file: file.to_string(),
      },
      expected_rule,
      rule_found: token.as_rule(),
    });
  }

  Ok(())
}

#[cfg(test)]
mod test {

  use std::str::FromStr;
  use std::vec;

  use crate::parser::*;
  use cuentitos_common::{
    Block, BlockSettings, Condition, FrequencyModifier, Modifier, Requirement, VariableKind,
  };

  use rand::distributions::Alphanumeric;
  use rand::{self, Rng};

  const SPECIAL_CHARACTERS: [&str; 11] = [".", "_", "/", ",", ";", "'", " ", "?", "!", "¿", "¡"];

  const COMPARISON_OPERATORS: [&str; 7] = ["=", "!=", "<", ">", "<=", ">=", "!"];

  const MODIFIER_OPERATORS: [&str; 5] = ["+", "-", "*", "/", "="];

  const INDENTATIONS: [&str; 1] = ["  "];

  const RESERVED_KEYWORDS: [&str; 9] = ["req", "freq", "mod", "->", "`", "*", "#", "##", "//"];

  #[test]
  fn parse_database_from_path_correctly() {
    parse_database_from_path("../examples/story-example.cuentitos").unwrap();
    //TODO: compare with fixture
  }

  #[test]
  fn parse_database_correctly() {
    let unparsed_file = include_str!("../../examples/story-example.cuentitos");
    let unparsed_config = include_str!("../../examples/cuentitos.toml");
    let config = Config::from_str(unparsed_config).unwrap();
    parse_database_str(unparsed_file, &config).unwrap();
    //TODO: compare with fixture
  }

  #[test]
  fn buckets_chance_must_sum_1() {
    let snake_case = make_random_snake_case();

    let frequency_1 = rand::thread_rng().gen_range(i8::MIN as f32..i8::MAX as f32);
    let child_1 = make_random_string();
    let mut frequency_2 = rand::thread_rng().gen_range(i8::MIN as f32..i8::MAX as f32);
    while frequency_1 + frequency_1 == 1. {
      frequency_2 = rand::thread_rng().gen_range(i8::MIN as f32..i8::MAX as f32);
    }

    let child_2 = make_random_string();
    let named_bucket_string = format!(
      "[{}]\n  ({}){}\n  ({}){}",
      snake_case, frequency_1, child_1, frequency_2, child_2
    );

    let named_bucket = parse_block_str(&named_bucket_string).unwrap_err();

    assert_eq!(
      named_bucket,
      PalabritasError::BucketSumIsNot1(
        Script {
          line: 1,
          col: 1,
          file: String::default()
        },
        snake_case
      )
    );
  }

  #[test]
  fn buckets_cant_have_frequency_and_chance() {
    let snake_case = make_random_snake_case();
    let frequency = rand::thread_rng().gen_range(1..100);
    let child_1 = make_random_string();
    let chance: f32 = rand::thread_rng().gen_range(0. ..1.);
    let child_2 = make_random_string();
    let named_bucket_string = format!(
      "[{}]\n  ({}){}\n  ({}){}",
      snake_case, frequency, child_1, chance, child_2
    );

    let named_bucket = parse_block_str(&named_bucket_string).unwrap_err();

    assert_eq!(
      named_bucket,
      PalabritasError::BucketHasFrequenciesAndChances(
        Script {
          line: 1,
          col: 1,
          file: String::default()
        },
        snake_case
      )
    );
  }

  #[test]
  fn buckets_cant_have_missing_probabilities() {
    let snake_case = make_random_snake_case();
    let frequency = rand::thread_rng().gen_range(1..100);
    let child_1 = make_random_string();
    let child_2 = make_random_string();
    let named_bucket_string = format!(
      "[{}]\n  ({}){}\n  {}",
      snake_case, frequency, child_1, child_2
    );

    let named_bucket = parse_block_str(&named_bucket_string).unwrap_err();

    assert_eq!(
      named_bucket,
      PalabritasError::BucketMissingProbability(
        Script {
          line: 3,
          col: 1,
          file: String::default()
        },
        child_2
      )
    );
  }
  #[test]
  fn parse_named_bucket_correctly() {
    //NamedBucket = { "[" ~ " "* ~ Probability? ~ SnakeCase ~ " "* ~ "]" }

    let float = rand::thread_rng().gen_range(i8::MIN as f32..i8::MAX as f32);
    let chance_string = format!("({})", float);

    let snake_case = make_random_snake_case();

    let frequency_1 = rand::thread_rng().gen_range(1..100);
    let child_1 = make_random_string();
    let frequency_2 = rand::thread_rng().gen_range(1..100);
    let child_2 = make_random_string();
    let named_bucket_string = format!(
      "[{} {}]\n  ({}){}\n  ({}){}",
      chance_string, snake_case, frequency_1, child_1, frequency_2, child_2
    );

    let named_bucket = parse_named_bucket_str(&named_bucket_string).unwrap();

    let chance = parse_chance_str(&chance_string).unwrap();

    let expected_value = Block::Bucket {
      name: Some(snake_case.clone()),
      settings: BlockSettings {
        chance: chance.clone(),
        ..Default::default()
      },
    };
    assert_eq!(named_bucket, expected_value);

    let blocks = parse_block_str(&named_bucket_string).unwrap();

    let expected_value = Block::Bucket {
      name: Some(snake_case),
      settings: BlockSettings {
        chance,
        children: vec![0, 1],
        ..Default::default()
      },
    };

    assert_eq!(blocks[0][0], expected_value);
  }

  #[test]
  fn parse_unnamed_bucket_correctly() {
    let parent = make_random_string();
    let child_1 = make_random_string();
    let float: f32 = rand::thread_rng().gen();
    let probabiliy_1 = format!("({})", float);
    let child_2 = make_random_string();
    let float = 1. - float;
    let probabiliy_2 = format!("({})", float);
    let block_string = format!(
      "{}\n  {}{}\n  {}{}",
      parent, probabiliy_1, child_1, probabiliy_2, child_2
    );

    let blocks = parse_block_str(&block_string).unwrap();

    let expected_text = Block::Text {
      id: parent,
      settings: BlockSettings {
        children: vec![0],
        ..Default::default()
      },
    };

    let expected_bucket = Block::Bucket {
      name: None,
      settings: BlockSettings {
        children: vec![0, 1],
        ..Default::default()
      },
    };

    assert_eq!(blocks[0][0], expected_text);
    assert_eq!(blocks[1][0], expected_bucket);
  }

  #[test]
  fn parse_choice_correctly() {
    //Choice = { "*" ~ " "* ~ Probability? ~ String }

    let float = rand::thread_rng().gen_range(i8::MIN as f32..i8::MAX as f32);
    let chance_string = format!("({})", float);
    let chance = parse_chance_str(&chance_string).unwrap();

    let string = make_random_string();
    let choice_string = format!("*{} {}", chance_string, string);
    let choice = parse_choice_str(&choice_string).unwrap();

    let expected_settings = BlockSettings {
      chance,
      ..Default::default()
    };
    let expected_value = Block::Choice {
      id: string,
      settings: expected_settings,
    };
    assert_eq!(choice, expected_value);
  }

  #[test]
  fn parse_section_correctly() {
    //Section = {"#" ~ " "* ~ Identifier ~ " "* ~ Command* ~ NewLine ~ ( NewLine | NewBlock | Subsection )* }

    let identifier = make_random_snake_case();

    let section_string = format!("#{}\n", identifier);
    let section = parse_section_str(&section_string, &Config::default()).unwrap();

    let expected_value = Block::Section {
      id: identifier.clone(),
      settings: BlockSettings {
        section: Some(Section {
          name: identifier,
          parent: None,
        }),
        ..Default::default()
      },
    };
    assert_eq!(section, expected_value);
  }
  #[test]
  fn parse_section_with_subsection_correctly() {
    //Section = {"#" ~ " "* ~ Identifier ~ " "* ~ Command* ~ NewLine ~ ( NewLine | NewBlock | Subsection )* }

    let section_identifier = make_random_snake_case();

    let subsection_identifier_1 = make_random_snake_case();
    let subsection_identifier_2 = make_random_snake_case();

    let section_string = format!(
      "#{}\n##{}\n##{}",
      section_identifier, subsection_identifier_1, subsection_identifier_2
    );

    let token = parse_str(&section_string, Rule::Section).unwrap();
    let mut parsing_data = ParsingData::default();
    parse_section(token, &mut parsing_data, 0).unwrap();

    let section_block = parsing_data.blocks[0][0].clone();

    let section = Section {
      name: section_identifier.clone(),
      parent: None,
    };
    let expected_value = Block::Section {
      id: section_identifier.clone(),
      settings: BlockSettings {
        section: Some(section.clone()),
        children: vec![0, 1],
        ..Default::default()
      },
    };
    assert_eq!(section_block, expected_value);

    let subsection_1_block = parsing_data.blocks[1][0].clone();

    let subsection_1 = Section {
      name: subsection_identifier_1.clone(),
      parent: Some(Box::new(section.clone())),
    };

    let expected_value = Block::Section {
      id: subsection_identifier_1.clone(),
      settings: BlockSettings {
        section: Some(subsection_1.clone()),
        script: Script {
          file: String::default(),
          line: 2,
          col: 1,
        },
        ..Default::default()
      },
    };
    assert_eq!(subsection_1_block, expected_value);

    let subsection_2_block = parsing_data.blocks[1][1].clone();

    let subsection_2 = Section {
      name: subsection_identifier_2.clone(),
      parent: Some(Box::new(section.clone())),
    };

    let expected_value = Block::Section {
      id: subsection_identifier_2.clone(),
      settings: BlockSettings {
        section: Some(subsection_2.clone()),
        script: Script {
          file: String::default(),
          line: 3,
          col: 1,
        },
        ..Default::default()
      },
    };

    assert_eq!(subsection_2_block, expected_value);
    let token = parse_str(&section_string, Rule::Database).unwrap();
    let parsing_data = ParsingData::default();
    let database = parse_database(token, parsing_data).unwrap();

    let id = *database.sections.get(&section).unwrap();
    assert_eq!(id, 0);

    let id = *database.sections.get(&subsection_1).unwrap();
    assert_eq!(id, 2);

    let id = *database.sections.get(&subsection_2).unwrap();
    assert_eq!(id, 3);
  }
  #[test]
  fn parse_text_correctly() {
    //Text = { Probability? ~ String }

    let float = rand::thread_rng().gen_range(0 as f32..1 as f32);
    let chance_string = format!("({})", float);

    let string = make_random_string();

    let text_string = format!("{} {}", chance_string, string);
    let text = parse_text_str(&text_string).unwrap();
    let chance = parse_chance_str(&chance_string).unwrap();

    let expected_settings = BlockSettings {
      chance,
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

    // //Command = {NEWLINE ~ (Indentation | " ")* ~ (Requirement | Frequency | Modifier) }
    let mut block_settings = BlockSettings::default();
    let mut config = Config::default();

    //Modifier
    let variable = make_random_identifier();
    let value = rand::thread_rng().gen::<f32>().to_string();
    let modifier_string = format!("\n set {} {}", variable, value);
    config
      .variables
      .insert(variable.clone(), VariableKind::Float);

    let expected_modifier = Modifier {
      variable,
      value,
      operator: ModifierOperator::Set,
    };

    block_settings.modifiers.push(expected_modifier);

    let token = parse_str(&modifier_string, Rule::Command).unwrap();
    add_command_to_block(token, &mut block, &config, &String::default()).unwrap();

    //Unique

    block_settings.unique = true;
    let token = parse_str("\n unique", Rule::Command).unwrap();
    add_command_to_block(token, &mut block, &config, &String::default()).unwrap();

    //Frequency

    let variable = make_random_identifier();
    let condition_string = variable.clone() + " " + &make_random_identifier();
    config.variables.insert(variable, VariableKind::String);

    let condition = parse_condition_str(&condition_string, &config).unwrap();
    let change_value: i32 = rand::thread_rng().gen();

    config
      .variables
      .insert(condition.variable.clone(), VariableKind::String);

    let frequency_string = format!("\n freq {} {}", condition_string, change_value);
    let expected_frequency = FrequencyModifier {
      condition,
      value: change_value,
    };

    block_settings.frequency_modifiers.push(expected_frequency);

    let token = parse_str(&frequency_string, Rule::Command).unwrap();
    add_command_to_block(token, &mut block, &config, &String::default()).unwrap();

    //Requirement

    let variable = make_random_identifier();
    let condition_string = variable.clone() + " " + &make_random_identifier();
    config.variables.insert(variable, VariableKind::String);

    let condition = parse_condition_str(&condition_string, &config).unwrap();

    config
      .variables
      .insert(condition.variable.clone(), VariableKind::String);

    let requirement_string = format!("\n req {}", condition_string);
    let expected_requirement = Requirement { condition };

    block_settings.requirements.push(expected_requirement);

    let token = parse_str(&requirement_string, Rule::Command).unwrap();
    add_command_to_block(token, &mut block, &config, &String::default()).unwrap();

    let expected_block = Block::Text {
      id: text_id,
      settings: block_settings,
    };

    assert_eq!(block, expected_block);
  }

  #[test]
  fn parse_tag_correctly() {
    //Tag = {"tag" ~ " "+ ~ Identifier}
    let tag_name = make_random_identifier();
    let tag_string = format!("tag {}", tag_name);
    let tag = parse_tag_str(&tag_string).unwrap();

    assert_eq!(tag, tag_name);
  }
  #[test]
  fn frequency_out_of_bucket_throws_error() {
    let text = make_random_string();
    let frequency: u32 = rand::thread_rng().gen_range(0..100);
    let database_str = format!("({}) {}", frequency, text);
    let err = parse_database_str(&database_str, &Config::default()).unwrap_err();
    assert_eq!(
      err,
      PalabritasError::FrequencyOutOfBucket(
        Script {
          line: 1,
          col: 1,
          file: String::default()
        },
        database_str
      )
    );

    let choice_str = make_random_string();
    let frequency: u32 = rand::thread_rng().gen_range(0..100);
    let database_str = format!("*({}) {}", frequency, choice_str);
    let err = parse_database_str(&database_str, &Config::default()).unwrap_err();
    assert_eq!(
      err,
      PalabritasError::FrequencyOutOfBucket(
        Script {
          line: 1,
          col: 1,
          file: String::default()
        },
        database_str
      )
    );

    let text = make_random_string();
    let choice_str_1 = make_random_string();
    let frequency_1: u32 = rand::thread_rng().gen_range(0..100);
    let choice_str_2 = make_random_string();
    let frequency_2: u32 = rand::thread_rng().gen_range(0..100);
    let database_str = format!(
      "{}\n  *({}) {}\n  *({}) {}",
      text, frequency_1, choice_str_1, frequency_2, choice_str_2
    );
    parse_database_str(&database_str, &Config::default()).unwrap();
  }

  #[test]
  fn frequency_modifier_without_frequency_chance_throws_error() {
    let text = make_random_string();
    let variable = make_random_identifier();
    let value = rand::thread_rng().gen::<f32>().to_string();
    let frequency: u32 = rand::thread_rng().gen_range(0..100);

    let mut config = Config::default();
    config
      .variables
      .insert(variable.clone(), VariableKind::Float);

    let database_str = format!("{}\n  freq {} {} {}", text, variable, value, frequency);
    let err = parse_database_str(&database_str, &config).unwrap_err();
    assert_eq!(
      err,
      PalabritasError::FrequencyModifierWithoutFrequencyChance(
        Script {
          line: 1,
          col: 1,
          file: String::default()
        },
        database_str
      )
    );
  }

  #[test]
  fn division_by_zero_throws_error() {
    let identifier = make_random_identifier();
    let tag_string = format!("set {} / 0", identifier);
    let mut variables = HashMap::default();
    variables.insert(identifier, VariableKind::Integer);

    let mut config = Config {
      variables,
      ..Default::default()
    };
    let value: PalabritasError = parse_modifier_str(&tag_string, &config).unwrap_err();

    assert_eq!(
      value,
      PalabritasError::DivisionByZero(
        Script {
          line: 1,
          col: 1,
          file: String::default()
        },
        tag_string
      )
    );

    let identifier = make_random_identifier();
    let tag_string = format!("set {} / 0", identifier);
    config.variables.insert(identifier, VariableKind::Float);

    let value: PalabritasError = parse_modifier_str(&tag_string, &config).unwrap_err();

    assert_eq!(
      value,
      PalabritasError::DivisionByZero(
        Script {
          line: 1,
          col: 1,
          file: String::default()
        },
        tag_string
      )
    );
  }

  #[test]
  fn duplicate_sections_throws_error() {
    let section_1 = make_random_identifier();
    let section_2 = section_1.clone();
    let err = parse_database_str(
      &format!("#{}\n#{}\n", section_1, section_2),
      &Config::default(),
    )
    .unwrap_err();

    let expected_error = PalabritasError::DuplicatedSection {
      first_appearance: Box::new(Script {
        file: String::default(),
        line: 1,
        col: 1,
      }),
      second_appearance: Box::new(Script {
        file: String::default(),
        line: 2,
        col: 1,
      }),
      section: Section {
        name: section_1,
        parent: None,
      },
    };

    assert_eq!(err, expected_error);
  }

  #[test]
  fn duplicate_subsections_throws_error() {
    let section_name = make_random_identifier();
    let subsection_name = make_random_identifier();
    let err = parse_database_str(
      &format!(
        "#{}\n##{}\n##{}\n",
        section_name, subsection_name, subsection_name
      ),
      &Config::default(),
    )
    .unwrap_err();

    let section = Section {
      name: section_name,
      parent: None,
    };
    let subsection = Section {
      name: subsection_name,
      parent: Some(Box::new(section.clone())),
    };
    let expected_error = PalabritasError::DuplicatedSection {
      first_appearance: Box::new(Script {
        file: String::default(),
        line: 2,
        col: 1,
      }),
      second_appearance: Box::new(Script {
        file: String::default(),
        line: 3,
        col: 1,
      }),
      section: subsection,
    };

    assert_eq!(err, expected_error);
  }

  #[test]
  fn subsections_named_after_sections_throws_error() {
    let section_1 = make_random_identifier();
    let subsection = make_random_identifier();
    let section_2 = subsection.clone();
    let err = parse_database_str(
      &format!("#{}\n##{}\n#{}\n", section_1, subsection, section_2),
      &Config::default(),
    )
    .unwrap_err();

    let expected_error = PalabritasError::SubsectioNamedAfterUpperSection {
      section_script: Box::new(Script {
        file: String::default(),
        line: 3,
        col: 1,
      }),
      subsection_script: Box::new(Script {
        file: String::default(),
        line: 2,
        col: 1,
      }),
      subsection: section_2,
    };

    assert_eq!(err, expected_error);
  }
  #[test]
  fn diverts_to_non_existent_sections_throws_error() {
    let error = parse_database_str("Text\n  ->Section", &Config::default()).unwrap_err();
    assert_eq!(
      error,
      PalabritasError::SectionDoesntExist {
        script: Script {
          line: 2,
          col: 3,
          file: String::default()
        },
        section: Section {
          name: "Section".to_string(),
          parent: None
        }
      }
    );

    parse_database_str(
      "#Secttion\n  Text\n    ->Subsection\n##Subsection\n  Subtext",
      &Config::default(),
    )
    .unwrap();
  }

  #[test]
  fn requirement_with_non_existent_variable_throws_error() {
    //Requirement = { "req" ~ " "+ ~ Condition ~ " "* }

    let error = parse_database_str("Text\n  req variable = value", &Config::default()).unwrap_err();
    assert_eq!(
      error,
      PalabritasError::VariableDoesntExist {
        script: Script {
          line: 2,
          col: 7,
          file: String::default()
        },
        string: "variable = value".to_string(),
        variable: "variable".to_string()
      }
    );
  }

  #[test]
  fn frequency_with_non_existent_variable_throws_error() {
    //Frequency = { "freq" ~ " "+ ~ Condition ~ " "+ ~ ( Float | Integer ) ~ " "* }

    let error =
      parse_database_str("Text\n  freq variable = value 10", &Config::default()).unwrap_err();
    assert_eq!(
      error,
      PalabritasError::VariableDoesntExist {
        script: Script {
          line: 2,
          col: 8,
          file: String::default()
        },
        string: "variable = value".to_string(),
        variable: "variable".to_string()
      }
    );
  }

  #[test]
  fn modifier_with_non_existent_variable_throws_error() {
    //Modifier = { "set" ~ " "+ ~ ( (Identifier ~ " "+ ~ ModifierOperator? ~ " "* ~ Value) | (NotOperator? ~ " "* ~ Identifier) ) ~ " "* }

    let error = parse_database_str("Text\n  set variable value", &Config::default()).unwrap_err();
    assert_eq!(
      error,
      PalabritasError::VariableDoesntExist {
        script: Script {
          line: 2,
          col: 3,
          file: String::default()
        },
        string: "set variable value".to_string(),
        variable: "variable".to_string()
      }
    );
  }

  #[test]
  fn integer_with_invalid_value_throws_error() {
    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::Integer);

    let script = Script {
      line: 0,
      col: 0,
      file: String::default(),
    };

    let error =
      check_variable_value_matches_type("variable", "hello", &config, &script, &String::default())
        .unwrap_err();

    let expected_error = PalabritasError::InvalidVariableValue {
      script: Box::new(script),
      variable: "variable".to_string(),
      value: "hello".to_string(),
      variable_type: VariableKind::Integer,
      string: String::default(),
    };

    assert_eq!(error, expected_error);
  }

  #[test]
  fn float_with_invalid_value_throws_error() {
    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::Float);

    let script = Script {
      line: 0,
      col: 0,
      file: String::default(),
    };

    let error =
      check_variable_value_matches_type("variable", "hello", &config, &script, &String::default())
        .unwrap_err();

    let expected_error = PalabritasError::InvalidVariableValue {
      script: Box::new(script),
      variable: "variable".to_string(),
      value: "hello".to_string(),
      variable_type: VariableKind::Float,
      string: String::default(),
    };

    assert_eq!(error, expected_error);
  }

  #[test]
  fn bool_with_invalid_value_throws_error() {
    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::Bool);

    let script = Script {
      line: 0,
      col: 0,
      file: String::default(),
    };

    let error =
      check_variable_value_matches_type("variable", "hello", &config, &script, &String::default())
        .unwrap_err();

    let expected_error = PalabritasError::InvalidVariableValue {
      script: Box::new(script),
      variable: "variable".to_string(),
      value: "hello".to_string(),
      variable_type: VariableKind::Bool,
      string: String::default(),
    };

    assert_eq!(error, expected_error);
  }

  #[test]
  fn enum_with_invalid_value_throws_error() {
    let mut config = Config::default();
    let kind = VariableKind::Enum(vec!["night".to_string(), "day".to_string()]);
    config
      .variables
      .insert("variable".to_string(), kind.clone());

    let script = Script {
      line: 0,
      col: 0,
      file: String::default(),
    };

    let error =
      check_variable_value_matches_type("variable", "hello", &config, &script, &String::default())
        .unwrap_err();

    let expected_error = PalabritasError::InvalidVariableValue {
      script: Box::new(script),
      variable: "variable".to_string(),
      value: "hello".to_string(),
      variable_type: kind,
      string: String::default(),
    };

    assert_eq!(error, expected_error);
  }

  #[test]
  fn requirement_with_invalid_variable_value_throws_error() {
    //Requirement = { "req" ~ " "+ ~ Condition ~ " "* }

    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::Integer);
    let error = parse_database_str("Text\n  req variable = value", &config).unwrap_err();
    assert_eq!(
      error,
      PalabritasError::InvalidVariableValue {
        script: Box::new(Script {
          line: 2,
          col: 7,
          file: String::default()
        }),
        string: "variable = value".to_string(),
        variable: "variable".to_string(),
        value: "value".to_string(),
        variable_type: VariableKind::Integer
      }
    );
  }

  #[test]
  fn frequency_with_invalid_variable_value_throws_error() {
    //Frequency = { "freq" ~ " "+ ~ Condition ~ " "+ ~ ( Float | Integer ) ~ " "* }
    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::Integer);
    let error = parse_database_str("Text\n  freq variable = value 10", &config).unwrap_err();
    assert_eq!(
      error,
      PalabritasError::InvalidVariableValue {
        script: Box::new(Script {
          line: 2,
          col: 8,
          file: String::default()
        }),
        string: "variable = value".to_string(),
        variable: "variable".to_string(),
        value: "value".to_string(),
        variable_type: VariableKind::Integer
      }
    );
  }

  #[test]
  fn modifier_with_invalid_variable_value_throws_error() {
    //Modifier = { "set" ~ " "+ ~ ( (Identifier ~ " "+ ~ ModifierOperator? ~ " "* ~ Value) | (NotOperator? ~ " "* ~ Identifier) ) ~ " "* }
    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::Integer);
    let error = parse_database_str("Text\n  set variable value", &config).unwrap_err();
    assert_eq!(
      error,
      PalabritasError::InvalidVariableValue {
        script: Box::new(Script {
          line: 2,
          col: 3,
          file: String::default()
        }),
        string: "set variable value".to_string(),
        variable: "variable".to_string(),
        value: "value".to_string(),
        variable_type: VariableKind::Integer
      }
    );
  }

  #[test]
  fn requirement_with_invalid_operator_throws_error() {
    //Requirement = { "req" ~ " "+ ~ Condition ~ " "* }

    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::String);
    let error = parse_database_str("Text\n  req variable > value", &config).unwrap_err();
    assert_eq!(
      error,
      PalabritasError::InvalidVariableOperator {
        script: Box::new(Script {
          line: 2,
          col: 7,
          file: String::default()
        }),
        string: "variable > value".to_string(),
        variable: "variable".to_string(),
        operator: ">".to_string(),
        variable_type: VariableKind::String
      }
    );
  }

  #[test]
  fn frequency_with_invalid_operator_throws_error() {
    //Frequency = { "freq" ~ " "+ ~ Condition ~ " "+ ~ ( Float | Integer ) ~ " "* }
    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::String);
    let error = parse_database_str("Text\n  freq variable > value 10", &config).unwrap_err();
    assert_eq!(
      error,
      PalabritasError::InvalidVariableOperator {
        script: Box::new(Script {
          line: 2,
          col: 8,
          file: String::default()
        }),
        string: "variable > value".to_string(),
        variable: "variable".to_string(),
        operator: ">".to_string(),
        variable_type: VariableKind::String
      }
    );
  }

  #[test]
  fn modifier_with_invalid_operator_throws_error() {
    //Modifier = { "set" ~ " "+ ~ ( (Identifier ~ " "+ ~ ModifierOperator? ~ " "* ~ Value) | (NotOperator? ~ " "* ~ Identifier) ) ~ " "* }
    let mut config = Config::default();
    config
      .variables
      .insert("variable".to_string(), VariableKind::String);
    let error = parse_database_str("Text\n  set variable +1", &config).unwrap_err();
    assert_eq!(
      error,
      PalabritasError::InvalidVariableOperator {
        script: Box::new(Script {
          line: 2,
          col: 3,
          file: String::default()
        }),
        string: "set variable +1".to_string(),
        variable: "variable".to_string(),
        operator: "+".to_string(),
        variable_type: VariableKind::String
      }
    );
  }

  #[test]
  fn parse_function_correctly() {
    //Function = {"`" ~ " "* ~ Identifier ~ (" " ~ Value)* ~ " "* ~ "`"}
    let name = make_random_identifier();
    let parameter_1 = make_random_identifier();
    let parameter_2 = make_random_identifier();
    let function_string = format!("`{} {} {}`", name, parameter_1, parameter_2);
    let value = parse_function_str(&function_string).unwrap();
    let expected_value = Function {
      name,
      parameters: vec![parameter_1, parameter_2],
    };

    assert_eq!(value, expected_value);
  }

  #[test]
  fn parse_divert_correctly() {
    //Divert = { "->"  ~ " "* ~ Identifier ~ ("/" ~ Identifier)? }
    //Section
    let section_name = make_random_identifier();
    let divert_string = format!("-> {}", section_name);
    let section = Section {
      name: section_name.clone(),
      parent: None,
    };
    let expected_value = Block::Divert {
      next: NextBlock::Section(section.clone()),
      settings: BlockSettings::default(),
    };

    let divert = parse_divert_str(&divert_string).unwrap();
    assert_eq!(divert, expected_value);

    //Subsection
    let subsection_name = make_random_identifier();
    let divert_string = format!("-> {}/{}", section_name, subsection_name);
    let subsection = Section {
      name: subsection_name.clone(),
      parent: Some(Box::new(section.clone())),
    };

    let expected_value = Block::Divert {
      next: NextBlock::Section(subsection.clone()),
      settings: BlockSettings::default(),
    };

    let divert = parse_divert_str(&divert_string).unwrap();
    assert_eq!(divert, expected_value);
  }

  #[test]
  fn parse_boomerang_divert_correctly() {
    //Section
    let section_name = make_random_identifier();
    let divert_string = format!("<-> {}", section_name);
    let section = Section {
      name: section_name.clone(),
      parent: None,
    };
    let expected_value = Block::BoomerangDivert {
      next: NextBlock::Section(section.clone()),
      settings: BlockSettings::default(),
    };

    let divert = parse_boomerang_divert_str(&divert_string).unwrap();
    assert_eq!(divert, expected_value);

    //Subsection
    let subsection_name = make_random_identifier();
    let divert_string = format!("<-> {}/{}", section, subsection_name);
    let subsection = Section {
      name: subsection_name.clone(),
      parent: Some(Box::new(section.clone())),
    };

    let expected_value = Block::BoomerangDivert {
      next: NextBlock::Section(subsection.clone()),
      settings: BlockSettings::default(),
    };

    let divert = parse_boomerang_divert_str(&divert_string).unwrap();
    assert_eq!(divert, expected_value);
  }

  #[test]
  fn parse_modifier_correctly() {
    //Modifier = { "set" ~ " "+ ~ ( (Identifier ~ " "+ ~ Value) | (NotOperator? ~ " "* ~ Identifier) ) ~ " "* }
    let variable = make_random_identifier();
    let value = rand::thread_rng().gen::<f32>().to_string();
    let modifier_string = format!("set {} {}", variable, value);

    let mut variables = HashMap::default();
    variables.insert(variable.clone(), VariableKind::Float);

    let config = Config {
      variables,
      ..Default::default()
    };

    let expected_value = Modifier {
      variable,
      value,
      operator: ModifierOperator::Set,
    };

    let modifier = parse_modifier_str(&modifier_string, &config).unwrap();

    assert_eq!(modifier, expected_value);
  }

  #[test]
  fn parse_frequency_correctly() {
    //Frequency = { "freq" ~ " "+ ~ Condition ~ " "+ ~ ( Float | Integer ) }
    let variable = make_random_identifier();
    let condition_string = variable.clone() + " " + &make_random_identifier();
    let mut variables = HashMap::default();
    variables.insert(variable, VariableKind::String);
    let config = Config {
      variables,
      ..Default::default()
    };

    let condition = parse_condition_str(&condition_string, &config).unwrap();
    let change_value: i32 = rand::thread_rng().gen();
    let frequency_string = format!("freq {} {}", condition_string, change_value);

    let expected_value = FrequencyModifier {
      condition,
      value: change_value,
    };
    let frequency = parse_frequency_str(&frequency_string, &config).unwrap();

    assert_eq!(frequency, expected_value);
  }

  #[test]
  fn parse_requirement_correctly() {
    //Requirement = { "req" ~ " "+ ~ Condition }
    let variable = make_random_identifier();
    let condition_string = variable.clone() + " " + &make_random_identifier();
    let mut variables = HashMap::default();
    variables.insert(variable, VariableKind::String);
    let config = Config {
      variables,
      ..Default::default()
    };

    let condition = parse_condition_str(&condition_string, &config).unwrap();
    let requirement_string = format!("req {}", condition_string);
    let expected_value = Requirement { condition };
    let requirement = parse_requirement_str(&requirement_string, &config).unwrap();

    assert_eq!(requirement, expected_value);
  }

  #[test]
  fn parse_condition_correctly() {
    /*Condition = { Identifier ~ " "* ~ (ComparisonOperator ~ " "*)? ~ Value } */
    let variable = make_random_identifier();

    let operator_string =
      COMPARISON_OPERATORS[rand::thread_rng().gen_range(0..COMPARISON_OPERATORS.len())];
    let operator = parse_comparison_operator_str(&operator_string).unwrap();

    let value: f32 = rand::thread_rng().gen();

    let condition_string = format!("{} {} {}", variable, operator_string, value);

    let mut variables = HashMap::default();
    variables.insert(variable.clone(), VariableKind::Float);
    let config = Config {
      variables,
      ..Default::default()
    };

    let expected_value = Condition {
      variable,
      operator: operator,
      value: value.to_string(),
    };

    let condition = parse_condition_str(&condition_string, &config).unwrap();
    assert_eq!(condition, expected_value);
  }

  #[test]
  fn parse_comparison_operator_correctly() {
    //ComparisonOperator = { "!=" | "=" | "<=" | ">=" | "<" | ">" | "!" }
    let operator = parse_comparison_operator_str("!=").unwrap();
    assert_eq!(operator, ComparisonOperator::NotEqual);

    let operator = parse_comparison_operator_str("=").unwrap();
    assert_eq!(operator, ComparisonOperator::Equal);

    let operator = parse_comparison_operator_str("<=").unwrap();
    assert_eq!(operator, ComparisonOperator::LessOrEqualThan);

    let operator = parse_comparison_operator_str(">=").unwrap();
    assert_eq!(operator, ComparisonOperator::GreaterOrEqualThan);

    let operator = parse_comparison_operator_str("<").unwrap();
    assert_eq!(operator, ComparisonOperator::LessThan);

    let operator = parse_comparison_operator_str(">").unwrap();
    assert_eq!(operator, ComparisonOperator::GreaterThan);

    let operator = parse_comparison_operator_str("!").unwrap();
    assert_eq!(operator, ComparisonOperator::NotEqual);
  }

  #[test]
  fn parse_modifier_operator_correctly() {
    //ModifierOperator = {"+" | "-" | "*" | "/" | "="}
    let operator = parse_modifier_operator_str("+").unwrap();
    assert_eq!(operator, ModifierOperator::Add);

    let operator = parse_modifier_operator_str("-").unwrap();
    assert_eq!(operator, ModifierOperator::Substract);

    let operator = parse_modifier_operator_str("*").unwrap();
    assert_eq!(operator, ModifierOperator::Multiply);

    let operator = parse_modifier_operator_str("/").unwrap();
    assert_eq!(operator, ModifierOperator::Divide);

    let operator = parse_modifier_operator_str("=").unwrap();
    assert_eq!(operator, ModifierOperator::Set);
  }

  #[test]
  fn parse_not_equal_condition_correctly() {
    /*Condition = { Identifier ~ " "* ~ (ComparisonOperator ~ " "*)? ~ Value } */
    let variable = make_random_identifier();
    let condition_string = format!("!{}", variable);

    let mut variables = HashMap::default();
    variables.insert(variable.clone(), VariableKind::Bool);
    let config = Config {
      variables,
      ..Default::default()
    };

    let expected_value = Condition {
      variable,
      operator: ComparisonOperator::NotEqual,
      value: "true".to_string(),
    };

    let condition = parse_condition_str(&condition_string, &config).unwrap();
    assert_eq!(condition, expected_value);
  }

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
  fn parse_modifier_operator_rule() {
    //ModifierOperator = {"+" | "-" | "*" | "/" | "="}
    for operator in MODIFIER_OPERATORS {
      assert_parse_rule(Rule::ModifierOperator, operator);
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
    let variable = make_random_identifier();
    let condition = variable + " " + &make_random_identifier();
    assert_parse_rule(Rule::Requirement, &("req ".to_string() + &condition));
  }
  #[test]
  fn parse_frequency_rule() {
    //frequency = { "freq" ~ " "+ ~ condition ~ " "+ ~ ( float | integer ) }
    let condition = make_random_condition_str();
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
    //Modifier = { "set" ~ " "+ ~ ( (Identifier ~ " "+ ~ ModifierOperator? ~ " "* ~ Value) | (NotOperator? ~ " "* ~ Identifier) ) ~ " "* }

    let identifier = make_random_identifier();
    let value = make_random_identifier();

    assert_parse_rule(Rule::Modifier, &format!("set {} {}", &identifier, &value));

    for operator in MODIFIER_OPERATORS {
      assert_parse_rule(
        Rule::Modifier,
        &format!("set {} {} {}", &identifier, operator, &value),
      );
    }

    assert_parse_rule(Rule::Modifier, &format!("set !{}", &identifier));
  }

  #[test]
  fn parse_divert_rule() {
    //divert = { "->"  ~ " "* ~ identifier ~ ("." ~ identifier)? }
    let section = make_random_identifier();
    let subsection = make_random_identifier();

    assert_parse_rule(Rule::Divert, &("->".to_string() + &section));
    assert_parse_rule(
      Rule::Divert,
      &("->".to_string() + &section + "/" + &subsection),
    );
  }

  #[test]
  fn parse_boomerang_divert_rule() {
    //BoomerangDivert = { Chance? ~ "<->"  ~ " "* ~ Identifier ~ ("/" ~ Identifier)? }
    let section = make_random_identifier();
    let subsection = make_random_identifier();

    assert_parse_rule(Rule::BoomerangDivert, &("<->".to_string() + &section));
    assert_parse_rule(
      Rule::BoomerangDivert,
      &("<->".to_string() + &section + "/" + &subsection),
    );
  }

  #[test]
  fn parse_command_rule() {
    //Command = {NEWLINE ~ Indentation* ~ (Requirement | Frequency | Modifier | Divert) }
    let requirement = "\nreq ".to_string() + &(make_random_condition_str());
    assert_parse_rule(Rule::Command, &(requirement));

    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    let frequency = "\nfreq ".to_string() + &make_random_condition_str() + " " + &integer;
    assert_parse_rule(Rule::Command, &(frequency));

    let modifier =
      "\nset ".to_string() + &make_random_identifier() + " " + &make_random_identifier();
    assert_parse_rule(Rule::Command, &(modifier));
  }

  #[test]
  fn parse_chance_rule() {
    //probability = { "(" ~ " "* ~ ( percentage | float | integer ) ~ " "* ~ ")" ~ " "* }
    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse_rule(Rule::Chance, &("(".to_string() + &percentage + ")"));

    let float = rand::thread_rng()
      .gen_range(i8::MIN as f32..i8::MAX as f32)
      .to_string();
    assert_parse_rule(Rule::Chance, &("(".to_string() + &float + ")"));

    let integer = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string();
    assert_parse_rule(Rule::Chance, &("(".to_string() + &integer + ")"));
  }

  #[test]
  fn parses_file_that_starts_with_a_section() {
    assert_parse_rule(Rule::Database, format!("# intro\n\n# previa\n").as_str());
  }

  #[test]
  fn parse_section_rule() {
    //Section = {"#"~ " "*  ~ Chance? ~ " "* ~ !ReservedKeywords ~ Identifier ~ " "* ~ Command* ~ NewLine ~ ( NewLine | NewBlock | Subsection )* }
    let identifier = make_random_identifier();
    assert_parse_rule(Rule::Section, &("#".to_string() + &identifier));
    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse_rule(
      Rule::Section,
      &(format!("# ({}) {}", percentage, identifier)),
    );
  }

  #[test]
  fn parse_subsection_rule() {
    //Subsection = { "##"~ " "*  ~ Chance? ~ " "* ~ !ReservedKeywords ~ Identifier ~ " "* ~ Command* ~ (NewLine | EOI) ~ (NewBlock | NewLine)*}
    let identifier = make_random_identifier();
    assert_parse_rule(Rule::Subsection, &("##".to_string() + &identifier));
    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse_rule(
      Rule::Subsection,
      &(format!("## ({}) {}", percentage, identifier)),
    );
  }
  #[test]
  fn parse_subsubsection_rule() {
    //Subsubsection = { Indentation* ~ "###"~ " "*  ~ Chance? ~ " "* ~ !ReservedKeywords ~ Identifier ~ " "* ~ Command* ~ (NewLine | EOI) ~ (NewBlock | NewLine | Subsubsubsection)*}
    let identifier = make_random_identifier();
    assert_parse_rule(Rule::Subsubsection, &("###".to_string() + &identifier));
    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse_rule(
      Rule::Subsubsection,
      &(format!("### ({}) {}", percentage, identifier)),
    );
  }
  #[test]
  fn parse_subsubsubsection_rule() {
    //Subsubsection = { Indentation* ~ "###"~ " "*  ~ Chance? ~ " "* ~ !ReservedKeywords ~ Identifier ~ " "* ~ Command* ~ (NewLine | EOI) ~ (NewBlock | NewLine | Subsubsubsection)*}
    let identifier = make_random_identifier();
    assert_parse_rule(Rule::Subsubsubsection, &("####".to_string() + &identifier));
    let percentage = rand::thread_rng().gen_range(i8::MIN..i8::MAX).to_string() + "%";
    assert_parse_rule(
      Rule::Subsubsubsection,
      &(format!("#### ({}) {}", percentage, identifier)),
    );
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

  #[test]
  fn parse_section_with_chance() {
    //Section = {"#"~ " "*  ~ Chance? ~ " "* ~ !ReservedKeywords ~ Identifier ~ " "* ~ Command* ~ NewLine ~ ( NewLine | NewBlock | Subsection )* }
    let identifier = make_random_identifier();
    let percentage = rand::thread_rng().gen_range(0..100);
    let section_str = format!("# ({}%) {}", percentage, identifier);
    let section = parse_section_str(&section_str, &Config::default()).unwrap();

    let expected_value = Block::Section {
      id: identifier.clone(),
      settings: BlockSettings {
        section: Some(Section {
          name: identifier,
          parent: None,
        }),
        chance: Chance::Probability(percentage as f32 / 100.),
        ..Default::default()
      },
    };

    assert_eq!(section, expected_value);
  }

  #[test]
  fn parse_subsection_with_chance() {
    //Subsection = { "##"~ " "*  ~ Chance? ~ " "* ~ !ReservedKeywords ~ Identifier ~ " "* ~ Command* ~ (NewLine | EOI) ~ (NewBlock | NewLine)*}
    let identifier = make_random_identifier();
    let percentage = rand::thread_rng().gen_range(0..100);
    let subsection_str = format!("## ({}%) {}", percentage, identifier);

    let token = parse_str(&subsection_str, Rule::Subsection).unwrap();
    let mut parsing_data = ParsingData::new(Config::default(), String::default());
    parse_subsection(token, &mut parsing_data, 0).unwrap();

    let subsection = parsing_data.blocks[0][0].clone();
    let expected_value = Block::Section {
      id: identifier.clone(),
      settings: BlockSettings {
        chance: Chance::Probability(percentage as f32 / 100.),
        ..Default::default()
      },
    };

    assert_eq!(subsection, expected_value);
  }

  #[test]
  fn parse_section_commands_correctly() {
    let identifier = make_random_snake_case();

    let mut config = Config::default();
    config
      .variables
      .insert("test".to_string(), VariableKind::Bool);
    let section = parse_section_str(&format!("# {}\n  req test\n", identifier), &config).unwrap();

    let expected_value = Block::Section {
      id: identifier.clone(),
      settings: BlockSettings {
        section: Some(Section {
          name: identifier,
          parent: None,
        }),
        requirements: vec![Requirement {
          condition: Condition {
            variable: "test".to_string(),
            operator: ComparisonOperator::Equal,
            value: "true".to_string(),
          },
        }],
        ..Default::default()
      },
    };
    assert_eq!(section, expected_value);
  }

  #[test]
  fn parse_database_rule() {
    //File = { SOI ~ (NEWLINE | BlockBlock | Knot )* ~ EOI }
    let unparsed_file = include_str!("../../examples/story-example.cuentitos");
    assert_parse_rule(Rule::Database, &unparsed_file);
  }

  #[test]
  fn parse_unique_rule() {
    //Unique = {"unique"}
    assert_parse_rule(Rule::Unique, "unique");
  }

  #[test]
  fn parse_tag_rule() {
    //Tag = {"tag" ~ " "+ ~ Identifier}
    let identifier = make_random_identifier();
    assert_parse_rule(Rule::Tag, &format!("tag {}", identifier));
  }

  #[test]
  fn parse_function_rule() {
    //Function = {"`" ~ " "* ~ Identifier ~ (" " ~ Value)* ~ " "* ~ "`"}
    let function = make_random_identifier();
    assert_parse_rule(Rule::Function, &format!("`{}`", function));
    let parameter_1 = make_random_identifier();
    let parameter_2 = make_random_identifier();
    assert_parse_rule(
      Rule::Function,
      &format!("`{} {} {}`", function, parameter_1, parameter_2),
    );
  }

  #[test]
  fn parse_modifiers_with_empty_lines() {
    let text = make_random_string();
    let variable_1 = make_random_identifier();
    let variable_2 = make_random_identifier();
    let modifier_1 = format!("set {}", variable_1);
    let modifier_2 = format!("set {}", variable_2);
    let mut config = Config::default();

    config
      .variables
      .insert(variable_1.clone(), VariableKind::Bool);
    config
      .variables
      .insert(variable_2.clone(), VariableKind::Bool);

    let database = parse_database_str(
      &format!("{}\n  {}\n\n  {}", text, modifier_1, modifier_2),
      &config,
    )
    .unwrap();

    let modifier_1 = Modifier {
      variable: variable_1,
      value: true.to_string(),
      operator: ModifierOperator::Set,
    };

    let modifier_2 = Modifier {
      variable: variable_2,
      value: true.to_string(),
      operator: ModifierOperator::Set,
    };

    let text_block = Block::Text {
      id: text,
      settings: BlockSettings {
        modifiers: vec![modifier_1, modifier_2],
        ..Default::default()
      },
    };

    let eof = Block::Divert {
      next: NextBlock::EndOfFile,
      settings: BlockSettings::default(),
    };

    let expected_database = Database {
      blocks: vec![text_block, eof],
      config,
      ..Default::default()
    };

    assert_eq!(database, expected_database);
  }

  #[test]
  fn subsections_can_start_with_indentation() {
    let identifier = make_random_identifier();
    assert_parse_rule(Rule::Subsection, &format!("  ## {}", identifier));
  }
  #[test]
  fn choices_with_empty_lines() {
    let text = make_random_string();
    let choice_text_1 = make_random_identifier();
    let choice_text_2 = make_random_identifier();
    let choice_1 = format!("* {}", choice_text_1);
    let choice_2 = format!("* {}", choice_text_2);

    let database = parse_database_str(
      &format!("{}\n  {}\n\n  {}", text, choice_1, choice_2),
      &Config::default(),
    )
    .unwrap();

    let choice_1 = Block::Choice {
      id: choice_text_1,
      settings: BlockSettings {
        script: Script {
          file: "".to_string(),
          line: 2,
          col: 3,
        },
        ..Default::default()
      },
    };

    let choice_2 = Block::Choice {
      id: choice_text_2,
      settings: BlockSettings {
        script: Script {
          file: "".to_string(),
          line: 4,
          col: 3,
        },
        ..Default::default()
      },
    };

    let text_block = Block::Text {
      id: text,
      settings: BlockSettings {
        children: vec![2, 3],
        ..Default::default()
      },
    };

    let eof = Block::Divert {
      next: NextBlock::EndOfFile,
      settings: BlockSettings::default(),
    };

    let expected_database = Database {
      blocks: vec![text_block, eof, choice_1, choice_2],
      ..Default::default()
    };

    assert_eq!(database, expected_database);
  }

  fn assert_parse_rule(rule: Rule, input: &str) {
    let pair = PalabritasParser::parse(rule, input)
      .expect("unsuccessful parse")
      .next()
      .unwrap();
    assert_eq!(pair.as_rule(), rule);
    assert_eq!(pair.as_str(), input);
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

    snake_case = snake_case.to_lowercase();

    for keyword in RESERVED_KEYWORDS {
      while snake_case.starts_with(keyword) {
        snake_case = make_random_snake_case();
      }
    }

    snake_case
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
    string = string.trim().to_string();
    for keyword in RESERVED_KEYWORDS {
      while string.starts_with(keyword) {
        string = make_random_string();
      }
    }

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

  fn parse_block_str(input: &str) -> Result<Vec<Vec<Block>>, PalabritasError> {
    let token = parse_str(input, Rule::Block)?;
    let mut parsing_data = ParsingData::default();
    parse_block(token, &mut parsing_data, 0)?;
    Ok(parsing_data.blocks)
  }
  fn make_random_condition_str() -> String {
    make_random_identifier() + " " + &make_random_identifier()
  }
}
