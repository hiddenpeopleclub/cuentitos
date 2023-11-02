use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use crate::Config;

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
  pub fn from_config(config: &Config) -> Self {
    let mut i18n = I18n {
      locales: config.locales.clone(),
      default_locale: config.default_locale.clone(),
      ..Default::default()
    };
    for locale in &i18n.locales {
      i18n
        .strings
        .insert(locale.to_string(), LanguageDb::default());
    }

    i18n
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
