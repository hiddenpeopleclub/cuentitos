use regex::Regex;
use cuentitos_common::ItemKind;
use core::str::Lines;
use cuentitos_common::ItemBuilder;
use cuentitos_common::Config;
use crate::parsable::Parsable;

#[derive(Default)]
pub struct Item;

impl Parsable<cuentitos_common::Item> for Item {
  fn parse<S>(content: S, _: &Config) -> Result<cuentitos_common::Item, String>
  where
    S: AsRef<str>,
  {
    let mut builder = ItemBuilder::new();
    let mut lines = content.as_ref().lines();

    builder.name(lines.next().unwrap());
    builder.description(lines.next().unwrap());

    let (kind, count) = parse_kind_and_count(lines);
    
    builder.kind(kind);
    builder.max_stack_count(count);

    Ok(builder.build())
  }
}

fn parse_kind_and_count(mut lines: Lines) -> (ItemKind, u8) {
  let ingredient_regexp = Regex::new(r"^ingredient").unwrap();
  let count_regexp = Regex::new(r"^(\d)").unwrap();

  let mut result = (ItemKind::Other, 0);

  if let Some(first) = lines.next() {
    if ingredient_regexp.is_match(first) {
      result.0 = ItemKind::Ingredient;
      if let Some(second) = lines.next() {
        if let Some(count) = count_regexp.captures_iter(second).next() {
          result.1 = count[1].parse::<u8>().unwrap();
        }        
      }
    } else {
      if let Some(count) = count_regexp.captures_iter(first).next() {
        result.1 = count[1].parse::<u8>().unwrap();
      }    
    }

    if ingredient_regexp.is_match(first) {
      result.0 = ItemKind::Ingredient;
    }

  }
  

  result
}

#[cfg(test)]
mod test {
  use cuentitos_common::{Config, ItemKind};
  use crate::{parsers::Item, parsable::Parsable};

  #[test]
  fn parses_title_and_description() {
    let content = include_str!("../../fixtures/items/00-moon-wolf.item");
    let item = Item::parse(content, &Config::default()).unwrap();
    assert_eq!(item.name, "Moon Wolf");
    assert_eq!(item.description, "A woolf with blue eyes that shine at night");
  }

  #[test]
  fn parses_with_default_kind_and_stack_count() {
    let content = include_str!("../../fixtures/items/00-moon-wolf.item");
    let item = Item::parse(content, &Config::default()).unwrap();
    assert_eq!(item.kind, ItemKind::Other);
    assert_eq!(item.max_stack_count, 0);
  }

  #[test]
  fn parses_with_specified_kind_and_stack_count() {
    let content = include_str!("../../fixtures/items/01-mushroom.item");
    let item = Item::parse(content, &Config::default()).unwrap();
    assert_eq!(item.kind, ItemKind::Ingredient);
    assert_eq!(item.max_stack_count, 6);
  }

  #[test]
  fn parses_only_max_stack_count_() {
    let content = include_str!("../../fixtures/items/02-dirt.item");
    let item = Item::parse(content, &Config::default()).unwrap();
    assert_eq!(item.kind, ItemKind::Other);
    assert_eq!(item.max_stack_count, 6)
  }

}
