use crate::{
  Divert, FloatProbability, Frequency, Modifier, PercentageProbability, Probability, Requirement,
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

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub enum ContentType {
  #[default]
  Text,
  NamedBucket,
  Choice,
}
