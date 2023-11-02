use core::fmt;
use serde::{Deserialize, Serialize};
use std::{fmt::Display, str::FromStr};
pub type SectionName = String;

#[derive(Debug, Default, Serialize, Deserialize, Eq, PartialEq, Clone, Hash)]
pub struct Section {
  pub name: SectionName,
  pub parent: Option<Box<Section>>,
}

impl Section {
  pub fn contains_subsection(&self, subsection: &Section) -> bool {
    if subsection == self {
      return true;
    } else if let Some(parent) = &subsection.parent {
      return self.contains_subsection(parent);
    }

    false
  }

  pub fn is_child_of(&self, section: &Section) -> bool {
    section.contains_subsection(self)
  }
}

/// Errors that can occur when deserializing a type.
#[derive(Debug, PartialEq, Eq, Clone)]
pub enum Error {
  EmptyString,
}

impl std::error::Error for Error {}

impl Display for Error {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    write!(f, "str provided is empty",)
  }
}

impl FromStr for Section {
  type Err = Error;
  fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
    let section_names = s.split('/');
    let mut section = None;
    for name in section_names {
      section = Some(Box::new(Section {
        name: name.to_string(),
        parent: section,
      }));
    }
    match section {
      Some(section) => Ok(*section),
      None => Err(Error::EmptyString),
    }
  }
}

impl fmt::Display for Section {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    let mut key = String::new();

    if let Some(parent) = &self.parent {
      key.push_str(&(parent.to_string() + "/"));
    }
    key.push_str(&self.name);

    write!(f, "{}", key)
  }
}
