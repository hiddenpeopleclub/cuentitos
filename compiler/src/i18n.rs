use std::path::Path;

use cuentitos_common::Database;

type Record = (String, String);

pub struct I18n {}

impl I18n {
  pub fn process<T, U>(
    database: &mut Database,
    base_path: T,
    destination_path: U,
  ) -> std::io::Result<()>
  where
    T: AsRef<Path>,
    U: AsRef<Path>,
  {
    // Load existing locales
    let mut base_path = base_path.as_ref().to_path_buf();
    base_path.pop();
    base_path.push("locales");

    for locale in &database.i18n.locales {
      if locale != &database.i18n.default_locale {
        let mut path = base_path.clone();
        path.push(format!("{}.csv", locale));
        if path.is_file() {
          let mut reader = csv::Reader::from_path(path)?;

          if let Some(db) = database.i18n.strings.get_mut(locale) {
            for result in reader.deserialize() {
              let record: Record = result?;
              if !record.0.is_empty() && !record.1.is_empty() {
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

    path.push("locales");
    std::fs::create_dir_all(&path)?;
    path.push(format!("{}.csv", database.i18n.default_locale));

    let mut wtr = csv::Writer::from_path(path)?;
    wtr.write_record(["Id", "Text"])?;

    for record in database.i18n.strings[&database.i18n.default_locale].iter() {
      wtr.serialize(record)?;
    }

    for record in &database.config.other_texts {
      wtr.serialize(record)?;
    }

    Ok(())
  }
}
