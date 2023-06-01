use crate::{
  Divert, FloatProbability, Frequency, Modifier, OutputText, PercentageProbability, Probability,
  Readable, Requirement,
};
use serde::{Deserialize, Serialize};

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct Content {
  pub content: Vec<Content>,
  pub probability: Option<Box<dyn Probability>>,
  pub text: String,
  pub content_type: ContentType,
  pub requirements: Vec<Requirement>,
  pub frequency_changes: Vec<Frequency>,
  pub modifiers: Vec<Modifier>,
  pub divert: Vec<Divert>,
  #[serde(skip)]
  pub cursor: usize,
}

impl PartialEq for Content {
  fn eq(&self, other: &Self) -> bool {
    return self.content == other.content
      && self.text == other.text
      && self.content_type == other.content_type
      && self.requirements == other.requirements
      && self.frequency_changes == other.frequency_changes
      && self.modifiers == other.modifiers
      && self.divert == other.divert
      && compare_probability::<FloatProbability>(self, other)
      && compare_probability::<PercentageProbability>(self, other);

    fn compare_probability<T>(content_1: &Content, content_2: &Content) -> bool
    where
      T: 'static,
      T: PartialEq,
    {
      if content_1.probability.is_some() != content_2.probability.is_some() {
        return true;
      }

      if content_1.probability.is_some() && content_2.probability.is_some() {
        if let Some(self_probability) = content_1
          .probability
          .as_ref()
          .unwrap()
          .as_any()
          .downcast_ref::<T>()
        {
          if let Some(other_probability) = content_2
            .probability
            .as_ref()
            .unwrap()
            .as_any()
            .downcast_ref::<T>()
          {
            if other_probability != self_probability {
              return false;
            }
          } else {
            return false;
          }
        }
      }
      true
    }
  }
}

impl Readable for Content {
  fn get_next_output(&mut self) -> Option<OutputText> {
    if self.cursor == 0 && self.content_type != ContentType::Text {
      self.cursor += 1;
    }

    if self.cursor > self.content.len() {
      return None;
    }

    if self.cursor == 0 {
      let output = Some(OutputText {
        text: self.text.clone(),
        choices: self.get_choices(),
      });
      self.cursor += 1;
      return output;
    }

    if !self.content[self.cursor - 1].meets_requirements() {
      self.cursor += 1;
      return self.get_next_output();
    }

    let output = self.content[self.cursor - 1].get_next_output();
    self.cursor += 1;

    output
  }

  fn pick_choice(&mut self, choice: usize) -> Option<OutputText> {
    if !self.is_in_choice() {
      return None;
    }

    let output = self.content[self.cursor - 1].pick_choice(choice);

    if output.is_none()
      && self.cursor + choice - 1 < self.content.len()
      && self.content[self.cursor + choice - 1].content_type == ContentType::Choice
    {
      self.cursor += choice;
      return self.get_next_output();
    }

    output
  }

  fn is_in_choice(&self) -> bool {
    if self.cursor == 0 || self.cursor > self.content.len() {
      return false;
    }

    if self.content[self.cursor - 1].content_type == ContentType::Choice {
      return true;
    }
    self.content[self.cursor - 1].is_in_choice()
  }
}

impl Content {
  pub fn meets_requirements(&self) -> bool {
    for requirement in &self.requirements {
      if !requirement.meets_requirement() {
        return false;
      }
    }
    true
  }

  pub fn get_choices(&self) -> Vec<String> {
    let mut choices = Vec::default();

    for i in 0..self.content.len() {
      if self.content[i].content_type == ContentType::Choice {
        choices.push(self.content[i].text.clone());
      } else {
        break;
      }
    }

    choices
  }
}
#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ContentType {
  #[default]
  Text,
  NamedBucket,
  Choice,
}
