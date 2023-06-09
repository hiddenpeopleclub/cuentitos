use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub type I18nId = String;
pub type LanguageDb = HashMap<I18nId, String>;
pub type LanguageId = String;

#[derive(Debug, Default, Serialize, Deserialize, Clone, Eq, PartialEq)]
pub struct I18n {
  pub locales: Vec<LanguageId>,
  pub default_locale: LanguageId,
  pub strings: HashMap<LanguageId, LanguageDb>,
}

impl I18n {
  pub fn has_locale(&self, locale: &String) -> bool {
    self.locales.contains(locale)
  }

  pub fn get_translation(&self, locale: &str, text: &str) -> String {
    if let Some(strings) = self.strings.get(locale) {
      match strings.get(text) {
        Some(t) => t.to_string(),
        None => format!("MISSING TRANSLATION `{}` in locale `{}`", text, locale),
      }
    } else {
      format!("MISSING LOCALE `{}`", locale)
    }
  }
}