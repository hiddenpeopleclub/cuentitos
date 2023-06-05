use palabritas_common::Block;
use palabritas_common::File;
use palabritas_common::OutputText;
use rand_pcg::Pcg32;
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize, PartialEq)]
pub struct Runtime {
  pub file: File,
  pub block_stack: Vec<Block>,
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

  pub fn next_output(&mut self) -> Option<OutputText> {
    if self.file.blocks.is_empty() {
      return None;
    }
    if self.block_stack.is_empty() {
      self.block_stack.push(self.file.blocks[0].clone());
    }

    let output = Runtime::get_block_output(self.block_stack.last().unwrap());
    self.update_navigation();

    output
  }

  fn get_block_output(block: &Block) -> Option<OutputText> {
    match block {
      Block::Text {
        i18n_id,
        navigation: _,
        settings: _,
      } => {
        return Some(OutputText {
          text: i18n_id.clone(),
          choices: Vec::default(),
        });
      }
      _ => {}
    }

    None
  }

  fn update_navigation(&mut self) {
    if let Some(navigation) = self.block_stack.clone().last().unwrap().get_navigation() {
      if !navigation.children.is_empty() && navigation.children[0] < self.file.blocks.len() {
        self
          .block_stack
          .push(self.file.blocks[navigation.children[0]].clone());
        return;
      }
      self.block_stack.pop();

      if self.block_stack.is_empty()
        && navigation.next.is_some()
        && navigation.next.unwrap() < self.file.blocks.len()
      {
        self
          .block_stack
          .push(self.file.blocks[navigation.next.unwrap()].clone());
        return;
      }

      println!(
        "{},{},{} < {}",
        self.block_stack.is_empty(),
        navigation.next.is_some(),
        navigation.next.unwrap(),
        self.file.blocks.len()
      );

      if let Some(previous_navigation) = self.file.blocks.last().unwrap().get_navigation() {
        let child_index = previous_navigation
          .children
          .iter()
          .position(|&r| r == navigation.index)
          .unwrap();
        if child_index + 1 < previous_navigation.children.len()
          && previous_navigation.children[child_index + 1] < self.file.blocks.len()
        {
          self
            .block_stack
            .push(self.file.blocks[previous_navigation.children[child_index + 1]].clone());
        }
      }
    }
  }

  pub fn pick_choice(&mut self, _choice: usize) -> Option<OutputText> {
    None
  }
}
