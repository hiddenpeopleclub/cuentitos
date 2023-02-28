use cuentitos_common::LanguageDb;
use crate::{Parser};

type Record = (String, String);

pub struct I18n {}

impl I18n {
  pub fn process(parser: &mut Parser) -> std::io::Result<cuentitos_common::I18n> {
    let mut i18n = cuentitos_common::I18n::default();
    i18n.default_locale = parser.config.default_locale.clone();
    i18n.locales = parser.config.locales.clone();

    for locale in &i18n.locales {
      i18n.strings.insert(locale.to_string(), LanguageDb::default());
    }

    // Setup default locale
    for (id, event) in &mut parser.events {
      if let Ok(event) = event {
        let base = format!("event-{}", id);
        
        if let Some(db) = i18n.strings.get_mut(&i18n.default_locale) {
          let key = format!("{}-title", base);
          db.insert(
            key.clone(), 
            event.title.clone()
          );
          event.title = key;

          let key = format!("{}-description", base);
          db.insert(
            key.clone(), 
            event.description.clone()
          );
          event.description = key;
          
          for (c_idx, choice) in event.choices.iter_mut().enumerate() {
            let key = format!("{}-choice-{}",base, c_idx);
            db.insert(
              key.clone(), 
              choice.text.clone()
            );
            choice.text = key;

            for (r_idx, result) in choice.results.iter_mut().enumerate() {
              let key = format!("{}-choice-{}-result-{}",base, c_idx, r_idx);
              db.insert(
                key.clone(),
                result.text.clone()
              );
              result.text = key;
            }
          }
        }
      }
    }

    // Load existing locales
    let mut base_path = parser.config.base_path.to_path_buf();
    base_path.push("locales");

    for locale in &i18n.locales {
      if locale != &i18n.default_locale {
        let mut path = base_path.clone();
        path.push(format!("{}.csv", locale));

        if path.is_file() {
          let mut reader = csv::Reader::from_path(path)?;

          if let Some(db) = i18n.strings.get_mut(locale) {
            for result in reader.deserialize() {
                let record: Record = result?;
                if record.0 != "" && record.1 != "" {
                  db.insert(record.0, record.1);  
                }
            }
          }
          
        }
      }
    }

    // Generate main translation file
    let mut path = parser.config.destination_path.to_path_buf();

    if path.extension() != None {
      path = path.parent().unwrap().to_path_buf();
    }

    std::fs::create_dir_all(&path)?;

    path.push(format!("{}.csv", i18n.default_locale));

    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(&["Id", "Text"])?;

    for record in i18n.strings[&i18n.default_locale].iter() {
      wtr.serialize(record)?;
    }

    Ok(i18n)
  }
}

#[cfg(test)]
mod test {
  use std::path::Path;
  use crate::I18n;
  use crate::Parser;
  use crate::Config;

  #[test]
  fn process_creates_i18n_file_if_not_present() {
    let config = Config::load("fixtures", "/tmp/fixtures-build").unwrap();
    let mut parser = Parser::new(config);
    parser.parse().unwrap();
    
    let i18n = I18n::process(&mut parser).unwrap();
    
    // Default Locale sets values
    assert_eq!(i18n.get_translation("en", "event-05-modifiers-title"), "A Basic Event with mods".to_string());
    assert_eq!(i18n.get_translation("en", "event-05-modifiers-description"), "This event has options with results and mods".to_string());
    assert_eq!(i18n.get_translation("en", "event-05-modifiers-choice-0"), "An Option".to_string());
    assert_eq!(i18n.get_translation("en", "event-05-modifiers-choice-0-result-0"), "One Result".to_string());

    // Loaded Locale
    assert_eq!(i18n.get_translation("es", "event-01-basic-title"), "Un evento básico".to_string());
    assert_eq!(i18n.get_translation("es", "event-non-existent"), "MISSING TRANSLATION `event-non-existent` in locale `es`");

    // Missing Entry
    assert_eq!(i18n.get_translation("ru", "event-non-existent"), "MISSING TRANSLATION `event-non-existent` in locale `ru`");

    // Invalid Locale
    assert_eq!(i18n.get_translation("it", "event-non-existent"), "MISSING LOCALE `it`");

    // Translation CSV File
    assert!(Path::new("/tmp/fixtures-build/en.csv").is_file());

  }
}
