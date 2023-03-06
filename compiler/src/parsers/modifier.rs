use crate::Config;
use core::fmt::Display;
use core::str::FromStr;
use cuentitos_common::VariableKind::*;

pub struct Modifier;

impl Modifier {
  pub fn parse<T>(data: T, config: &Config) -> Result<cuentitos_common::Modifier, String>
  where
    T: AsRef<str>,
  {
    let params: Vec<&str> = data.as_ref().split(' ').collect();

    match params[0] {
      "var" => {
        let variable = params[1];
        match config.variables.get_key_value(variable) {
          Some((_, kind)) => {
            let result = match kind {
              Integer => Self::parse_amount::<&str, i32>(params[2]),
              Float => Self::parse_amount::<&str, f32>(params[2]),
              Bool => Self::parse_amount::<&str, bool>(params[2]),
            };

            match result {
              Ok(amount) => Ok(cuentitos_common::Modifier::Resource {
                id: variable.to_string(),
                amount,
              }),
              Err(error) => Err(format!("{} for variable '{}'", error, variable)),
            }
          }
          None => Err(format!(
            "\"{}\" is not defined as a valid variable",
            variable
          )),
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
            Err(error) => Err(format!("{} for reputation '{}'", error, id)),
          }
        } else {
          Err(format!("'{}' is not a valid reputation", id))
        }
      }
      "decision" => Ok(cuentitos_common::Modifier::Decision(params[1].to_string())),
      "achievement" => Ok(cuentitos_common::Modifier::Achievement(
        params[1].to_string(),
      )),
      _ => Err(format!("\"{}\" is not a valid modifier", params[0])),
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
  use cuentitos_common::Variable;
  use cuentitos_common::VariableKind::*;

  #[test]
  fn error_on_wrong_modifier() {
    let config = Config::default();
    let result = Modifier::parse("wrong health 100", &config);
    assert_eq!(
      Err("\"wrong\" is not a valid modifier".to_string()),
      result
    );
  }

  #[test]
  fn parses_integer_variable() {
    let mut config = Config::default();
    let id = "health".to_string();

    config.variables.insert(id.clone(), Integer);
    let variable = Variable {
      id: id.clone(),
      kind: Integer,
    };

    let result = Modifier::parse("var health 100", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        id: variable.id.clone(),
        amount: 100.to_string()
      }
    );

    let result = Modifier::parse("var health -100", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        id: variable.id.clone(),
        amount: (-100).to_string()
      }
    );
  }

  #[test]
  fn parses_float_variable() {
    let mut config = Config::default();
    let id = "health".to_string();

    config.variables.insert(id.clone(), Float);
    let variable = Variable {
      id: id.clone(),
      kind: Float,
    };

    let result = Modifier::parse("var health 0.9", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        id: variable.id.clone(),
        amount: 0.9.to_string()
      }
    );

    let result = Modifier::parse("var health -0.9", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        id: variable.id.clone(),
        amount: (-0.9).to_string()
      }
    );
  }

  #[test]
  fn parses_bool_variable() {
    let mut config = Config::default();
    let id = "health".to_string();

    config.variables.insert(id.clone(), Bool);
    let variable = Variable {
      id: id.clone(),
      kind: Bool,
    };

    let result = Modifier::parse("var health true", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        id: variable.id.clone(),
        amount: "true".to_string()
      }
    );

    let result = Modifier::parse("var health false", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::Modifier::Resource {
        id: variable.id.clone(),
        amount: "false".to_string()
      }
    );
  }

  #[test]
  fn error_on_missing_variable() {
    let config = Config::default();
    let result = Modifier::parse("var health 100", &config);
    assert_eq!(
      Err("\"health\" is not defined as a valid variable".to_string()),
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
  fn error_on_wrong_variable_value() {
    let mut config = Config::default();
    config.variables.insert("health".to_string(), Integer);
    let result = Modifier::parse("var health false", &config);
    assert_eq!(
      Err("Invalid value: 'false' for variable 'health'".to_string()),
      result
    );
  }
}
