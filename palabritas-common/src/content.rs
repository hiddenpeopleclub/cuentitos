use crate::{Divert, Frequency, Modifier, Probability, Requirement};

#[derive(Default)]
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

#[derive(Default)]
pub enum ContentType {
  #[default]
  Text,
  NamedBucket,
  Choice,
}
