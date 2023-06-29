use std::path::Path;

use cuentitos_common::{Database, LanguageDb};

type Record = (String, String);

pub struct I18n {}

impl I18n {
  pub fn process<T, U>(
    database: &Database,
    base_path: T,
    destination_path: U,
  ) -> std::io::Result<cuentitos_common::I18n>
  where
    T: AsRef<Path>,
    U: AsRef<Path>,
  {
    let mut i18n = cuentitos_common::I18n {
      locales: database.config.locales.clone(),
      default_locale: database.config.default_locale.clone(),
      ..Default::default()
    };

    for locale in &i18n.locales {
      i18n
        .strings
        .insert(locale.to_string(), LanguageDb::default());
    }

    // Setup default locale
    for block in &database.blocks {
      if let Some(i18n_id) = block.get_i18n_id() {
        if let Some(db) = i18n.strings.get_mut(&i18n.default_locale) {
          db.insert(i18n_id.clone(), i18n_id);
        }
      }
    }

    // Load existing locales
    let mut base_path = base_path.as_ref().to_path_buf();
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
              if record.0.is_empty() && record.1.is_empty() {
                db.insert(record.0, record.1);
              }
            }
          }
        }
      }
    }

    // Generate main translation file
    let mut path = destination_path.as_ref().to_path_buf();

    if path.extension().is_some() {
      path = path.parent().unwrap().to_path_buf();
    }

    std::fs::create_dir_all(&path)?;

    path.push(format!("{}.csv", i18n.default_locale));

    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["Id", "Text"])?;

    for record in i18n.strings[&i18n.default_locale].iter() {
      wtr.serialize(record)?;
    }

    Ok(i18n)
  }
}
