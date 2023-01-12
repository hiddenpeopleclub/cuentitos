use crate::Config;
use core::fmt::Display;
use core::str::FromStr;
use cuentitos_common::Resource;
use cuentitos_common::ResourceKind::*;

pub struct Modifier;

impl Modifier {
  pub fn parse<T>(data: T, config: &Config) -> Result<cuentitos_common::Modifier, String>
  where
    T: AsRef<str>,
  {
    let params: Vec<&str> = data.as_ref().split(' ').collect();

    match params[0] {
      "resource" => {
        let resource = params[1];
        match config.resources.get_key_value(resource) {
          Some((_, kind)) => {
            let result = match kind {
              Integer => Self::parse_amount::<&str, i32>(params[2]),
              Float => Self::parse_amount::<&str, f32>(params[2]),
              Bool => Self::parse_amount::<&str, bool>(params[2]),
            };

            match result {
              Ok(amount) => {
                let resource = Resource {
                  id: resource.to_string(),
                  kind: kind.clone(),
                };
                Ok(cuentitos_common::Modifier::Resource { resource, amount })
              }
              Err(error) => return Err(format!("{} for resource '{}'", error, resource)),
            }
          }
          None => {
            return Err(format!(
              "\"{}\" is not defined as a valid resource",
              resource
            ))
          }
        }
      }
      "item" => {
        // TODO(fran): find a way to check if the item is valid, should this be done in a separate validation step?
        let id = params[1].to_string();
        let mut amount = "1".to_string();
        if params.len() > 2 {
          amount = Self::parse_amount::<&str, i32>(params[2])?;
        }
        Ok(cuentitos_common::Modifier::Item { id, amount })
      }
      "reputation" => {
        let id = params[1].to_string();
        if config.reputations.contains(&id) {
          match Self::parse_amount::<&str, i32>(params[2]) {
            Ok(amount) => Ok(cuentitos_common::Modifier::Reputation { id, amount }),
            Err(error) => return Err(format!("{} for reputation '{}'", error, id)),
          }
        } else {
          return Err(format!("'{}' is not a valid reputation", id));
        }
      }
      "decision" => Ok(cuentitos_common::Modifier::Decision(params[1].to_string())),
      "achievement" => Ok(cuentitos_common::Modifier::Achievement(
        params[1].to_string(),
      )),
      _ => return Err(format!("\"{}\" is not a valid requirement", params[0])),
    }
  }

  fn parse_amount<T, U>(data: T) -> Result<String, String>
  where
    T: AsRef<str>,
    U: FromStr + Display,
    <U as FromStr>::Err: std::fmt::Display,
  {
    let value = data.as_ref().to_string();

    match U::from_str(&value) {
      Ok(value) => Ok(value.to_string()),
      Err(_) => Err(format!("Invalid value: '{}'", data.as_ref())),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::parsers::modifier::Modifier;
  use crate::Config;
  use cuentitos_common::Resource;
  use cuentitos_common::ResourceKind::*;

  #[test]
  fn error_on_wrong_requirement() {
    let config = Config::default();
    let result = Modifier::parse("wrong health 100", &config);
    assert_eq!(
      Err("\"wrong\" is not a valid requirement".to_string()),
      result
    );
  }

  #[test]
  fn parses_integer_resource() {
    let mut config = Config::default();
    let id = "health".to_string();

    config.resources.insert(id.clone(), Integer);
    let resource = Resource {
      id: id.clone(),
      kind: Integer,
    };

    let result = Modifier::parse("resource health 100", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        resource: resource.clone(),
        amount: 100.to_string()
      }
    );

    let result = Modifier::parse("resource health -100", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        resource: resource.clone(),
        amount: (-100).to_string()
      }
    );
  }

  #[test]
  fn parses_float_resource() {
    let mut config = Config::default();
    let id = "health".to_string();

    config.resources.insert(id.clone(), Float);
    let resource = Resource {
      id: id.clone(),
      kind: Float,
    };

    let result = Modifier::parse("resource health 0.9", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        resource: resource.clone(),
        amount: 0.9.to_string()
      }
    );

    let result = Modifier::parse("resource health -0.9", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        resource: resource.clone(),
        amount: (-0.9).to_string()
      }
    );
  }

  #[test]
  fn parses_bool_resource() {
    let mut config = Config::default();
    let id = "health".to_string();

    config.resources.insert(id.clone(), Bool);
    let resource = Resource {
      id: id.clone(),
      kind: Bool,
    };

    let result = Modifier::parse("resource health true", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        resource: resource.clone(),
        amount: "true".to_string()
      }
    );

    let result = Modifier::parse("resource health false", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        resource: resource.clone(),
        amount: "false".to_string()
      }
    );
  }

  #[test]
  fn error_on_missing_resource() {
    let config = Config::default();
    let result = Modifier::parse("resource health 100", &config);
    assert_eq!(
      Err("\"health\" is not defined as a valid resource".to_string()),
      result
    );
  }

  #[test]
  fn parses_items() {
    let config = Config::default();

    let result = Modifier::parse("item wooden_figure", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Item {
        id: "wooden_figure".to_string(),
        amount: 1.to_string()
      }
    );

    let result = Modifier::parse("item wooden_figure -3", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Item {
        id: "wooden_figure".to_string(),
        amount: (-3).to_string()
      }
    );
  }
  
  // TODO(fran): Figure out how to implement this
  // #[test]
  // fn error_on_missing_item() {
  //   let config = Config::default();
  //   let _result = Modifier::parse("item wood 1", &config).unwrap();
  //   // assert_eq!(Err("\"wood\" is not defined as a valid item".to_string()), result);
  //   todo!()
  // }

  #[test]
  fn parses_reputations() {
    let mut config = Config::default();
    config.reputations.push("friends".to_string());

    let result = Modifier::parse("reputation friends 1", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Reputation {
        id: "friends".to_string(),
        amount: 1.to_string()
      }
    );

    let result = Modifier::parse("reputation friends -1", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Reputation {
        id: "friends".to_string(),
        amount: (-1).to_string()
      }
    );
  }

  #[test]
  fn error_on_wrong_reputation() {
    let config = Config::default();
    let result = Modifier::parse("reputation friends =-5", &config);
    assert_eq!(
      Err("'friends' is not a valid reputation".to_string()),
      result
    );
  }

  #[test]
  fn parses_decision() {
    let config = Config::default();

    let result = Modifier::parse("decision a_decision", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Decision("a_decision".to_string())
    );
  }

  #[test]
  fn parses_achievement() {
    let config = Config::default();
    let result = Modifier::parse("achievement an_achievement", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Achievement("an_achievement".to_string())
    );
  }

  #[test]
  fn error_on_wrong_resource_value() {
    let mut config = Config::default();
    config.resources.insert("health".to_string(), Integer);
    let result = Modifier::parse("resource health false", &config);
    assert_eq!(
      Err("Invalid value: 'false' for resource 'health'".to_string()),
      result
    );
  }
}
