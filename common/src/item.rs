use crate::ItemId;
use serde::{Deserialize, Serialize};
use strum_macros::EnumIter;

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone, EnumIter)]
pub enum ItemKind {
  Ingredient,
  #[default]
  Other,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct Item {
  pub id: ItemId,
  pub kind: ItemKind,
  pub name: String,
  pub description: String,
  pub max_stack_count: u8,
}

#[derive(Default)]
pub struct ItemBuilder {
  item: Item,
}

impl ItemBuilder {
  pub fn new() -> ItemBuilder {
    ItemBuilder {
      ..Default::default()
    }
  }

  pub fn id<T>(&mut self, id: T) -> &mut ItemBuilder
  where
    T: AsRef<str>,
  {
    self.item.id = id.as_ref().to_string();
    self
  }

  pub fn kind(&mut self, kind: ItemKind) -> &mut ItemBuilder {
    self.item.kind = kind;
    self
  }

  pub fn name<T>(&mut self, name: T) -> &mut ItemBuilder
  where
    T: AsRef<str>,
  {
    self.item.name = name.as_ref().to_string();
    self
  }

  pub fn description<T>(&mut self, description: T) -> &mut ItemBuilder
  where
    T: AsRef<str>,
  {
    self.item.description = description.as_ref().to_string();
    self
  }

  pub fn max_stack_count(&mut self, count: u8) -> &mut ItemBuilder {
    self.item.max_stack_count = count;
    self
  }

  pub fn build(&mut self) -> Item {
    self.item.clone()
  }
}

#[cfg(test)]
mod test {
  use crate::item::*;

  #[test]
  fn item_builder_supports_id() {
    let item = ItemBuilder::new().build();
    assert_eq!(item.id, "");

    let item = ItemBuilder::new().id("my-item").build();

    assert_eq!(item.id, "my-item");
  }

  #[test]
  fn item_builder_supports_kind() {
    let item = ItemBuilder::new().build();
    assert_eq!(item.kind, ItemKind::Other);

    let item = ItemBuilder::new().kind(ItemKind::Ingredient).build();

    assert_eq!(item.kind, ItemKind::Ingredient);
  }

  #[test]
  fn item_builder_supports_name_and_description() {
    let item = ItemBuilder::new().build();
    assert_eq!(item.name, "");
    assert_eq!(item.description, "");

    let item = ItemBuilder::new()
      .name("my-item")
      .description("my description")
      .build();

    assert_eq!(item.name, "my-item");
    assert_eq!(item.description, "my description");
  }

  #[test]
  fn item_builder_supports_max_stack_count() {
    let item = ItemBuilder::new().build();
    assert_eq!(item.max_stack_count, 0);

    let item = ItemBuilder::new().max_stack_count(6).build();

    assert_eq!(item.max_stack_count, 6);
  }
}
