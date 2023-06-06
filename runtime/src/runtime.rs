use palabritas_common::Block;
use palabritas_common::File;
use palabritas_common::Navigation;
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

    self.update_navigation();

    self.get_block_output(self.block_stack.last().unwrap())
  }

  
  pub fn pick_choice(&mut self, choice: usize) -> Option<OutputText> {
    if self.file.blocks.is_empty() {
      return None;
    }

    let choices = self.get_choices(self.block_stack.last().unwrap().get_navigation().unwrap());

    if choices.is_empty() {
      println!("There are no choices");
      return None;
    }

    if choice >= choices.len() {
      println!("There's only {} options", choices.len());
      return None;
    }

    if choices[choice] >= self.file.blocks.len() {
      println!("Invalid option");
      return None;
    }

    self
      .block_stack
      .push(self.file.blocks[choices[choice]].clone());
    self.update_navigation();
    self.next_output()
  }

  fn get_block_output(&self, block: &Block) -> Option<OutputText> {
    if let Block::Text {
      i18n_id,
      navigation,
      settings: _,
    } = block
    {
      return Some(OutputText {
        text: i18n_id.clone(),
        choices: self.get_choices_strings(navigation),
      });
    }

    None
  }

  fn update_navigation(&mut self) {
    if self.block_stack.is_empty() {
      self.block_stack.push(self.file.blocks[0].clone());
      return;
    }

    if !self.update_navigation_to_first_child() {
      while !self.block_stack.is_empty() && !self.update_navigation_to_next_sibling() {}
    }
  }

  fn get_choices_strings(&self, navigation: &Navigation) -> Vec<String> {
    let mut choices = Vec::default();

    for child in &navigation.children {
      if *child < self.file.blocks.len() {
        if let Block::Choice {
          i18n_id,
          navigation: _,
          settings: _,
        } = &self.file.blocks[*child]
        {
          choices.push(i18n_id.clone())
        }
      }
    }
    choices
  }

  fn get_choices(&self, navigation: &Navigation) -> Vec<usize> {
    let mut choices = Vec::default();
    for child in &navigation.children {
      if *child < self.file.blocks.len() {
        if let Block::Choice {
          i18n_id: _,
          navigation: _,
          settings: _,
        } = &self.file.blocks[*child]
        {
          choices.push(*child)
        }
      }
    }
    choices
  }

  fn update_navigation_to_first_child(&mut self) -> bool {
    if let Some(navigation) = self.block_stack.clone().last().unwrap().get_navigation() {
      if !navigation.children.is_empty() && navigation.children[0] < self.file.blocks.len() {
        let first_child = self.file.blocks[navigation.children[0]].clone();
        match first_child {
          Block::Text {
            i18n_id: _,
            navigation: _,
            settings: _,
          } => {
            self.block_stack.push(first_child);
          }
          _ => {
            println!("Make a choice");
          }
        }
        return true;
      }
    }
    false
  }

  fn update_navigation_to_next_sibling(&mut self) -> bool {
    let binding = self.block_stack.clone();
    let navigation = binding.last().unwrap().get_navigation();

    if navigation.is_none() {
      return false;
    }

    let navigation = navigation.unwrap();

    self.block_stack.pop();

    if self.block_stack.is_empty()
    {
      if  navigation.next.is_some()
      && navigation.next.unwrap() < self.file.blocks.len()
      {
        self
        .block_stack
        .push(self.file.blocks[navigation.next.unwrap()].clone());
        return true;
      }
      return false;
    }

    if let Some(previous_navigation) = self.block_stack.last().unwrap().get_navigation() {
      let mut child_index = previous_navigation
        .children
        .iter()
        .position(|&r| r == navigation.index)
        .unwrap()
        + 1;
      while child_index < previous_navigation.children.len()
        && previous_navigation.children[child_index] < self.file.blocks.len()
      {
        let next_child = self.file.blocks[previous_navigation.children[child_index]].clone();
        if let Block::Text {
          i18n_id: _,
          navigation: _,
          settings: _,
        } = next_child
        {
          self.block_stack.push(next_child);
          return true;
        }
        child_index += 1;
      }
    }
    false
  }

}


#[cfg(test)]
mod test {
    use palabritas_common::{File, Block, Navigation, BlockSettings, OutputText};

    use crate::Runtime;


  #[test]
  fn new_runtime_works_correctly() {
    let file = File{
        blocks: vec![Block::None],
    };
    let runtime = Runtime::new(file.clone());
    assert_eq!(runtime.file, file);
  }

  #[test]
  fn get_choices_works_correctly() {

    let navigation = Navigation { children: vec![1,2,3], index: 0, next: None };
    let parent =  Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 1, next: None };
    let choice_1 = Block::Choice { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 2, next: None };
    let choice_2 = Block::Choice { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    let navigation = Navigation { children: Vec::default(), index: 3, next: None };
    let child_text  = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let file = File{
        blocks: vec![parent.clone(),choice_1,choice_2,child_text],
    };
    let runtime = Runtime::new(file.clone());
    let choices = runtime.get_choices(parent.get_navigation().unwrap());
    let expected_result = vec![1,2];
    assert_eq!(choices, expected_result);
  }
  #[test]
  fn get_choices_strings_works_correctly() {

    let navigation = Navigation { children: vec![1,2,3], index: 0, next: None };
    let parent =  Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 1, next: None };
    let choice_1 = Block::Choice { i18n_id: "a".to_string(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 2, next: None };
    let choice_2 = Block::Choice { i18n_id: "b".to_string(), navigation, settings: BlockSettings::default() };
    let navigation = Navigation { children: Vec::default(), index: 3, next: None };
    let child_text  = Block::Text { i18n_id: "c".to_string(), navigation, settings: BlockSettings::default() };

    let file = File{
        blocks: vec![parent.clone(),choice_1,choice_2,child_text],
    };
    let runtime = Runtime::new(file);
    let choices = runtime.get_choices_strings(parent.get_navigation().unwrap());
    let expected_result = vec!["a".to_string(),"b".to_string()];
    assert_eq!(choices, expected_result);
  }
  #[test]
  fn update_navigation_to_first_child_works_correctly() {

    let navigation = Navigation { children: vec![1,2], index: 0, next: None };
    let parent =  Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 1, next: None };
    let child_1 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 2, next: None };
    let child_2 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };


    let file = File{
      blocks: vec![parent.clone(),child_1.clone(),child_2.clone()],
    };

    let mut runtime = Runtime::new(file);
    runtime.block_stack.push(parent);
    assert!(runtime.update_navigation_to_first_child());
    assert_eq!(*runtime.block_stack.last().unwrap(),child_1);

  }

  #[test]
  fn update_navigation_to_next_sibling_works_correctly() {


    let navigation = Navigation { children: vec![1,2,3], index: 0, next: Some(4) };
    let parent =  Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 1, next: None };
    let child_1 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 2, next: None };
    let child_2 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: Vec::default(), index: 3, next: None };
    let child_3 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: Vec::default(), index: 4, next: None };
    let sibling =  Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let file = File{
      blocks: vec![parent.clone(),child_1.clone(),child_2.clone(), child_3.clone(), sibling.clone()],
    };

    let mut runtime = Runtime::new(file);
    runtime.block_stack.push(parent);
    runtime.block_stack.push(child_1);
    assert!(runtime.update_navigation_to_next_sibling());
    assert_eq!(*runtime.block_stack.last().unwrap(),child_2);
    assert!(runtime.update_navigation_to_next_sibling());
    assert_eq!(*runtime.block_stack.last().unwrap(),child_3);
    runtime.block_stack.pop();
    assert!(runtime.update_navigation_to_next_sibling());
    assert_eq!(*runtime.block_stack.last().unwrap(),sibling);

  }
  #[test]
  fn update_navigation_works_correctly()
  {

    let navigation = Navigation { children: vec![1,2,3], index: 0, next: Some(4) };
    let parent =  Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 1, next: None };
    let child_1 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 2, next: None };
    let child_2 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: Vec::default(), index: 3, next: None };
    let child_3 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: Vec::default(), index: 4, next: None };
    let sibling =  Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let file = File{
      blocks: vec![parent.clone(),child_1.clone(),child_2.clone(), child_3.clone(), sibling.clone()],
    };
    let mut runtime = Runtime::new(file);

    runtime.update_navigation();
    assert_eq!(*runtime.block_stack.last().unwrap(), parent);

    runtime.update_navigation();
    assert_eq!(*runtime.block_stack.last().unwrap(), child_1);

    runtime.update_navigation();
    assert_eq!(*runtime.block_stack.last().unwrap(), child_2);

    runtime.update_navigation();
    assert_eq!(*runtime.block_stack.last().unwrap(), child_3);

    runtime.update_navigation();
    assert_eq!(*runtime.block_stack.last().unwrap(), sibling);

  }

  #[test]
  fn get_block_output_works_correctly(){

    let navigation = Navigation { children: vec![1,2], index: 0, next: Some(4) };
    let parent =  Block::Text { i18n_id: "parent".to_string(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: vec![1,2,3], index: 0, next: Some(4) };
    let choice_1 =  Block::Choice { i18n_id: "1".to_string(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: vec![1,2,3], index: 0, next: Some(4) };
    let choice_2 =  Block::Choice { i18n_id: "2".to_string(), navigation, settings: BlockSettings::default() };

    let file = File{
      blocks: vec![parent.clone(),choice_1.clone(),choice_2],
    };
    let runtime = Runtime::new(file);

    let output = runtime.get_block_output(&parent);
    let expected_output = Some(OutputText{
      text: "parent".to_string(),
      choices: vec!["1".to_string(),"2".to_string()],
    });

    assert_eq!(output,expected_output);
  }

  #[test]
  fn next_output_works_correctly(){

    let navigation = Navigation { children: vec![1,2], index: 0, next: Some(4) };
    let parent =  Block::Text { i18n_id: "parent".to_string(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: vec![1,2,3], index: 0, next: Some(4) };
    let choice_1 =  Block::Choice { i18n_id: "1".to_string(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: vec![1,2,3], index: 0, next: Some(4) };
    let choice_2 =  Block::Choice { i18n_id: "2".to_string(), navigation, settings: BlockSettings::default() };

    let file = File{
      blocks: vec![parent.clone(),choice_1.clone(),choice_2.clone()],
    };
    let mut runtime = Runtime::new(file);

    let output = runtime.next_output();
    let expected_output = Some(OutputText{
      text: "parent".to_string(),
      choices: vec!["1".to_string(),"2".to_string()],
    });

    assert_eq!(output,expected_output);
    assert_eq!(runtime.block_stack,vec![parent]);
  }

  #[test]
  fn next_output_doesnt_work_with_empty_file(){
    let mut runtime = Runtime::new(File::default());
    assert_eq!(runtime.next_output(),None);
  }

  #[test]
  fn update_navigation_to_next_sibling_ignores_choices(){
    let navigation = Navigation { children: vec![1,2,3,4], index: 0, next: Some(5) };
    let parent =  Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 1, next: None };
    let child_1 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: Vec::default(), index: 2, next: None };
    let choice = Block::Choice { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 3, next: None };
    let child_2 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: Vec::default(), index: 4, next: None };
    let child_3 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let navigation = Navigation { children: Vec::default(), index: 5, next: None };
    let sibling =  Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };

    let file = File{
      blocks: vec![parent.clone(), child_1.clone(), choice, child_2.clone(), child_3.clone(), sibling.clone()],
    };

    let mut runtime = Runtime::new(file);
    runtime.block_stack.push(parent);
    runtime.block_stack.push(child_1);
    assert!(runtime.update_navigation_to_next_sibling());
    assert_eq!(*runtime.block_stack.last().unwrap(),child_2);
    assert!(runtime.update_navigation_to_next_sibling());
    assert_eq!(*runtime.block_stack.last().unwrap(),child_3);
    runtime.block_stack.pop();
    assert!(runtime.update_navigation_to_next_sibling());
    assert_eq!(*runtime.block_stack.last().unwrap(),sibling);

  }

  #[test]
  fn update_navigation_to_first_child_ignores_choices()
  {
    let navigation = Navigation { children: vec![1,2,3], index: 0, next: None };
    let parent =  Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 1, next: None };
    let choice = Block::Choice { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
  
    let navigation = Navigation { children: Vec::default(), index: 2, next: None };
    let child_1 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };
    
    let navigation = Navigation { children: Vec::default(), index: 3, next: None };
    let child_2 = Block::Text { i18n_id: String::default(), navigation, settings: BlockSettings::default() };


    let file = File{
      blocks: vec![parent.clone(), choice, child_1,child_2],
    };

    let mut runtime = Runtime::new(file);
    runtime.block_stack.push(parent.clone());
    assert!(runtime.update_navigation_to_first_child());
    assert_eq!(*runtime.block_stack.last().unwrap(),parent);
  }
}