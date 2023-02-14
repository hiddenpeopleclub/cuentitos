use std::collections::HashMap;
use serde::{Deserialize, Serialize};

pub type LanguageDb = HashMap<String, String>;
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
        Some(t) => return t.to_string(),
        None => return format!("MISSING TRANSLATION `{}` in locale `{}`", text, locale)
      };
    } else {
      return format!("MISSING LOCALE `{}`", locale);
    }
  }  
}
