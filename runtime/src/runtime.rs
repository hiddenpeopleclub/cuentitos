use palabritas_common::BlockId;
use palabritas_common::Definition;
use palabritas_common::File;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Block {
  pub text: String,
  pub choices: Vec<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Runtime {
  pub file: File,
  pub block_stack: Vec<BlockId>,
  #[serde(skip)]
  rng: Option<Pcg32>,
  seed: u64,
}

impl Runtime {
  pub fn new(file: File) -> Runtime {
    Runtime {
      file,
      ..Default::default()
    }
  }

  pub fn next_block(&mut self) -> Option<Block> {
    if self.file.blocks.is_empty() {
      return None;
    }

    self.update_stack();

    self.get_next_block_output()
  }

  pub fn pick_choice(&mut self, choice: usize) -> Option<Block> {
    if self.file.blocks.is_empty() {
      return None;
    }

    let choices = self.get_choices();

    if choices.is_empty() {
      println!("There are no choices");
      return None;
    }

    if choice >= choices.len() {
      println!("There's only {} options", choices.len());
      return None;
    }

    if choices[choice].0 >= self.file.blocks.len() {
      println!("Invalid option");
      return None;
    }

    self.block_stack.push(choices[choice].clone());
    self.next_block()
  }

  fn get_next_block_output(&self) -> Option<Block> {
    let id = self.block_stack.last().unwrap();
    let block = self.get_block(id);
    if let palabritas_common::Block::Text(Definition {
      i18n_id,
      navigation: _,
      settings: _,
    }) = block
    {
      return Some(Block {
        text: i18n_id.clone(),
        choices: self.get_choices_strings(),
      });
    }

    None
  }

  fn update_stack(&mut self) {
    if self.block_stack.is_empty() {
      self.block_stack.push(BlockId(0));
      return;
    }

    let last_block_id = self.block_stack.last().unwrap();

    if last_block_id.0 >= self.file.blocks.len() {
      return;
    }

    let last_block = self.get_block(last_block_id);

    if let Some(last_navigation) = last_block.get_navigation() {
      if !last_navigation.children.is_empty() {
        if let palabritas_common::Block::Choice(_) = self.get_block(&last_navigation.children[0]) {
          println!("Make a choice\n");
          return;
        }
        self.block_stack.push(last_navigation.children[0].clone());
        return;
      }
    }
    self.pop_stack_and_find_next();
  }

  fn pop_stack_and_find_next(&mut self) {
    let last_block_id = self.block_stack.last().unwrap().clone();

    let last_block = self.get_block(&last_block_id).clone();

    if let Some(last_navigation) = last_block.get_navigation() {
      match &last_navigation.next {
        palabritas_common::NavigationNext::None => {}
        palabritas_common::NavigationNext::BlockId(other_id) => {
          self.block_stack.pop();
          self.block_stack.push(other_id.clone());
          return;
        }
        palabritas_common::NavigationNext::EOF => {
          println!("Story finished\n");
          self.block_stack = vec![BlockId(0)];
          return;
        }
        palabritas_common::NavigationNext::Section(_) => todo!(),
      }
    }

    self.block_stack.pop();

    if self.block_stack.is_empty() {
      self.block_stack.push(BlockId(last_block_id.0 + 1));
      return;
    }

    if let Some(parent_navigation) =
      self.file.blocks[self.block_stack.last().unwrap().0].get_navigation()
    {
      let mut child_found = false;
      for child in &parent_navigation.children {
        if child_found {
          if let palabritas_common::Block::Choice(_) = self.get_block(child) {
            continue;
          }
          self.block_stack.push(child.clone());
          return;
        }
        if child.0 == last_block_id.0 {
          child_found = true;
        }
      }
    }
    self.pop_stack_and_find_next()
  }

  fn get_block(&self, id: &BlockId) -> &palabritas_common::Block {
    &self.file.blocks[id.0]
  }

  fn get_choices_strings(&self) -> Vec<String> {
    let choices = self.get_choices();

    let mut choices_strings = Vec::default();

    for choice in choices {
      if let palabritas_common::Block::Choice(Definition {
        i18n_id,
        navigation: _,
        settings: _,
      }) = self.get_block(&choice)
      {
        choices_strings.push(i18n_id.clone());
      }
    }

    choices_strings
  }

  fn get_choices(&self) -> Vec<BlockId> {
    let mut choices = Vec::default();

    if self.block_stack.is_empty() {
      return choices;
    }

    let last_block_id = self.block_stack.last().unwrap().clone();
    let last_block = self.get_block(&last_block_id).clone();

    let navigation = last_block.get_navigation();

    if navigation.is_none() {
      return choices;
    }

    let navigation = navigation.unwrap();

    for child in &navigation.children {
      if child.0 < self.file.blocks.len() {
        if let palabritas_common::Block::Choice(_) = self.get_block(child) {
          choices.push(child.clone())
        }
      }
    }
    choices
  }
}

#[cfg(test)]
mod test {

  use crate::Runtime;
  use palabritas_common::{
    Block, BlockId, BlockSettings, Definition, File, Navigation, NavigationNext,
  };

  #[test]
  fn new_runtime_works_correctly() {
    let file = File {
      blocks: vec![Block::None],
    };
    let runtime = Runtime::new(file.clone());
    assert_eq!(runtime.file, file);
  }

  #[test]
  fn get_choices_works_correctly() {
    let navigation = Navigation {
      children: vec![BlockId(1), BlockId(2), BlockId(3)],
      next: NavigationNext::None,
    };
    let parent = palabritas_common::Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let choice_1 = palabritas_common::Block::Choice(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let choice_2 = palabritas_common::Block::Choice(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });
    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let child_text = palabritas_common::Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let file = File {
      blocks: vec![parent.clone(), choice_1, choice_2, child_text],
    };
    let runtime = Runtime {
      file,
      block_stack: vec![BlockId(0)],
      ..Default::default()
    };

    let choices = runtime.get_choices();
    let expected_result = vec![BlockId(1), BlockId(2)];
    assert_eq!(choices, expected_result);
  }
  #[test]
  fn get_choices_strings_works_correctly() {
    let navigation: Navigation = Navigation {
      children: vec![BlockId(1), BlockId(2), BlockId(3)],
      next: NavigationNext::None,
    };
    let parent = Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let choice_1 = Block::Choice(Definition {
      i18n_id: "a".to_string(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let choice_2 = Block::Choice(Definition {
      i18n_id: "b".to_string(),
      navigation,
      settings: BlockSettings::default(),
    });
    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let child_text = Block::Text(Definition {
      i18n_id: "c".to_string(),
      navigation,
      settings: BlockSettings::default(),
    });

    let file = File {
      blocks: vec![parent.clone(), choice_1, choice_2, child_text],
    };
    let runtime = Runtime {
      file,
      block_stack: vec![BlockId(0)],
      ..Default::default()
    };
    let choices = runtime.get_choices_strings();
    let expected_result = vec!["a".to_string(), "b".to_string()];
    assert_eq!(choices, expected_result);
  }
  #[test]
  fn updates_stack_to_first_child_correctly() {
    let navigation = Navigation {
      children: vec![BlockId(1), BlockId(2)],
      next: NavigationNext::None,
    };
    let parent = Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let child_1 = Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let child_2 = Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let file = File {
      blocks: vec![parent.clone(), child_1.clone(), child_2.clone()],
    };

    let mut runtime = Runtime {
      file,
      block_stack: vec![BlockId(0)],
      ..Default::default()
    };
    runtime.update_stack();
    assert_eq!(*runtime.block_stack.last().unwrap(), BlockId(1));
  }

  #[test]
  fn update_stack_to_next_sibling_correctly() {
    let navigation = Navigation {
      children: vec![BlockId(2), BlockId(3), BlockId(4)],
      next: NavigationNext::None,
    };

    let parent = Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let sibling = Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let child_1 = Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let child_2 = Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: Vec::default(),
      next: NavigationNext::None,
    };
    let child_3 = Block::Text(Definition {
      i18n_id: String::default(),
      navigation,
      settings: BlockSettings::default(),
    });

    let file = File {
      blocks: vec![
        parent.clone(),
        sibling.clone(),
        child_1.clone(),
        child_2.clone(),
        child_3.clone(),
      ],
    };

    let mut runtime = Runtime {
      file,
      block_stack: vec![BlockId(0), BlockId(2)],
      ..Default::default()
    };

    runtime.update_stack();
    assert_eq!(*runtime.block_stack.last().unwrap(), BlockId(3));
    runtime.update_stack();
    assert_eq!(*runtime.block_stack.last().unwrap(), BlockId(4));
    runtime.update_stack();
    assert_eq!(*runtime.block_stack.last().unwrap(), BlockId(1));
  }

  #[test]
  fn get_next_block_output_works_correctly() {
    let navigation = Navigation {
      children: vec![BlockId(1), BlockId(2)],
      next: NavigationNext::None,
    };
    let parent = Block::Text(Definition {
      i18n_id: "parent".to_string(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: vec![BlockId(1), BlockId(2), BlockId(3)],
      next: NavigationNext::None,
    };
    let choice_1 = Block::Choice(Definition {
      i18n_id: "1".to_string(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: vec![BlockId(1), BlockId(2), BlockId(3)],
      next: NavigationNext::None,
    };
    let choice_2 = Block::Choice(Definition {
      i18n_id: "2".to_string(),
      navigation,
      settings: BlockSettings::default(),
    });

    let file = File {
      blocks: vec![parent.clone(), choice_1.clone(), choice_2],
    };

    let runtime = Runtime {
      file,
      block_stack: vec![BlockId(0)],
      ..Default::default()
    };

    let output = runtime.get_next_block_output();
    let expected_output = Some(crate::Block {
      text: "parent".to_string(),
      choices: vec!["1".to_string(), "2".to_string()],
    });

    assert_eq!(output, expected_output);
  }

  #[test]
  fn next_block_works_correctly() {
    let navigation = Navigation {
      children: vec![BlockId(1), BlockId(2)],
      next: NavigationNext::None,
    };
    let parent = Block::Text(Definition {
      i18n_id: "parent".to_string(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: vec![BlockId(1), BlockId(2), BlockId(3)],
      next: NavigationNext::None,
    };
    let choice_1 = Block::Choice(Definition {
      i18n_id: "1".to_string(),
      navigation,
      settings: BlockSettings::default(),
    });

    let navigation = Navigation {
      children: vec![BlockId(1), BlockId(2), BlockId(3)],
      next: NavigationNext::None,
    };
    let choice_2 = Block::Choice(Definition {
      i18n_id: "2".to_string(),
      navigation,
      settings: BlockSettings::default(),
    });

    let file = File {
      blocks: vec![parent.clone(), choice_1.clone(), choice_2.clone()],
    };
    let mut runtime = Runtime {
      file,
      ..Default::default()
    };

    let output = runtime.next_block();
    let expected_output = Some(crate::Block {
      text: "parent".to_string(),
      choices: vec!["1".to_string(), "2".to_string()],
    });

    assert_eq!(output, expected_output);
    assert_eq!(runtime.block_stack, vec![BlockId(0)]);
  }

  #[test]
  fn next_output_doesnt_work_with_empty_file() {
    let mut runtime = Runtime::new(File::default());
    assert_eq!(runtime.next_block(), None);
  }
}
