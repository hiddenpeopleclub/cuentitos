use crate::Config;
use core::fmt::Display;
use core::str::FromStr;
use cuentitos_common::Resource;
use cuentitos_common::{Condition, ResourceKind::*};

pub struct EventRequirement;

impl EventRequirement {
  pub fn parse<T>(data: T, config: &Config) -> Result<cuentitos_common::EventRequirement, String>
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
              Integer => {
                Self::parse_amount_and_condition::<&str, i32>(params[2], Condition::HigherThan)
              }
              Float => {
                Self::parse_amount_and_condition::<&str, f32>(params[2], Condition::HigherThan)
              }
              Bool => Self::parse_amount_and_condition::<&str, bool>(params[2], Condition::Equals),
            };

            match result {
              Ok((amount, condition)) => {
                let resource = Resource {
                  id: resource.to_string(),
                  kind: kind.clone(),
                };
                Ok(cuentitos_common::EventRequirement::Resource {
                  resource,
                  amount,
                  condition,
                })
              }
              Err(error) => Err(format!("{} for resource '{}'", error, resource)),
            }
          }
          None => Err(format!(
            "\"{}\" is not defined as a valid resource",
            resource
          )),
        }
      }
      "item" => {
        // TODO(fran): find a way to check if the item is valid, should this be done in a separate validation step?
        let id = params[1].to_string();
        let mut amount = "1".to_string();
        let mut condition = Condition::Equals;
        if params.len() > 2 {
          let result = Self::parse_amount_and_condition::<&str, u32>(params[2], Condition::Equals)?;
          amount = result.0;
          condition = result.1;
        }
        Ok(cuentitos_common::EventRequirement::Item {
          id,
          amount,
          condition,
        })
      }
      "reputation" => {
        let id = params[1].to_string();
        if config.reputations.contains(&id) {
          match Self::parse_amount_and_condition::<&str, i32>(params[2], Condition::HigherThan) {
            Ok((amount, condition)) => Ok(cuentitos_common::EventRequirement::Reputation {
              id,
              amount,
              condition,
            }),
            Err(error) => Err(format!("{} for reputation '{}'", error, id)),
          }
        } else {
          Err(format!("'{}' is not a valid reputation", id))
        }
      }
      "time_of_day" => {
        let mut id = params[1].to_string();
        let condition = Self::parse_condition(&mut id, Condition::Equals);
        let id = match id.as_str() {
          "morning" => cuentitos_common::TimeOfDay::Morning,
          "noon" => cuentitos_common::TimeOfDay::Noon,
          "evening" => cuentitos_common::TimeOfDay::Evening,
          "night" => cuentitos_common::TimeOfDay::Night,
          _ => return Err(format!("'{}' is not a valid time of day", id)),
        };
        Ok(cuentitos_common::EventRequirement::TimeOfDay { id, condition })
      }
      "event" => {
        let mut id = params[1].to_string();
        let condition = Self::parse_condition(&mut id, Condition::Depends);
        Ok(cuentitos_common::EventRequirement::Event { id, condition })
      }
      "decision" => {
        let mut id = params[1].to_string();
        let condition = Self::parse_condition(&mut id, Condition::Depends);
        Ok(cuentitos_common::EventRequirement::Decision { id, condition })
      }
      "tile" => {
        let mut id = params[1].to_string();
        let condition = Self::parse_condition(&mut id, Condition::Depends);
        if config.tiles.contains(&id) {
          Ok(cuentitos_common::EventRequirement::Tile { id, condition })
        } else {
          Err(format!("'{}' is not a valid tile", id))
        }
      }
      _ => Err(format!("\"{}\" is not a valid requirement", params[0])),
    }
  }

  fn parse_condition(id: &mut String, default: Condition) -> Condition {
    match id.chars().next() {
      Some(c) => {
        if c == '!' {
          id.remove(0);
          Condition::MutEx
        } else {
          default
        }
      }
      None => default,
    }
  }

  fn parse_amount_and_condition<T, U>(
    data: T,
    default_condition: Condition,
  ) -> Result<(String, Condition), String>
  where
    T: AsRef<str>,
    U: FromStr + Display,
  {
    let mut value = data.as_ref().to_string();
    let condition = match value.chars().next() {
      Some(c) => match c {
        '>' => {
          value.remove(0);
          Condition::HigherThan
        }
        '<' => {
          value.remove(0);
          Condition::LessThan
        }
        '=' => {
          value.remove(0);
          Condition::Equals
        }
        _ => default_condition,
      },
      None => return Err(format!("Invalid value: '{}'", data.as_ref())),
    };

    match U::from_str(&value) {
      Ok(value) => Ok((value.to_string(), condition)),
      Err(_) => Err(format!("Invalid value: '{}'", data.as_ref())),
    }
  }
}

#[cfg(test)]
mod test {
  use crate::parsers::event_requirement::EventRequirement;
  use crate::Config;
  use cuentitos_common::Condition::*;
  use cuentitos_common::Resource;
  use cuentitos_common::ResourceKind::*;

  #[test]
  fn error_on_wrong_requirement() {
    let config = Config::default();
    let result = EventRequirement::parse("wrong health 100", &config);
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
    let amount = "100".to_string();

    let result = EventRequirement::parse("resource health 100", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Resource {
        resource: resource.clone(),
        condition: HigherThan,
        amount: amount.clone()
      }
    );

    let result = EventRequirement::parse("resource health >100", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Resource {
        resource: resource.clone(),
        condition: HigherThan,
        amount: amount.clone()
      }
    );

    let result = EventRequirement::parse("resource health <100", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Resource {
        resource: resource.clone(),
        condition: LessThan,
        amount: amount.clone()
      }
    );

    let result = EventRequirement::parse("resource health =100", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Resource {
        resource: resource.clone(),
        condition: Equals,
        amount: amount.clone()
      }
    )
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
    let amount = "0.9".to_string();

    let result = EventRequirement::parse("resource health 0.9", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Resource {
        resource: resource.clone(),
        condition: HigherThan,
        amount: amount.clone()
      }
    );

    let result = EventRequirement::parse("resource health >0.9", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Resource {
        resource: resource.clone(),
        condition: HigherThan,
        amount: amount.clone()
      }
    );

    let result = EventRequirement::parse("resource health <0.9", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Resource {
        resource: resource.clone(),
        condition: LessThan,
        amount: amount.clone()
      }
    );

    let result = EventRequirement::parse("resource health =0.9", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Resource {
        resource: resource.clone(),
        condition: Equals,
        amount: amount.clone()
      }
    )
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

    let result = EventRequirement::parse("resource health true", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Resource {
        resource: resource.clone(),
        condition: Equals,
        amount: "true".to_string()
      }
    );

    let result = EventRequirement::parse("resource health false", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Resource {
        resource: resource.clone(),
        condition: Equals,
        amount: "false".to_string()
      }
    );
  }

  #[test]
  fn error_on_missing_resource() {
    let config = Config::default();
    let result = EventRequirement::parse("resource health 100", &config);
    assert_eq!(
      Err("\"health\" is not defined as a valid resource".to_string()),
      result
    );
  }

  #[test]
  fn parses_items() {
    let config = Config::default();

    let result = EventRequirement::parse("item wooden_figure", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Item {
        id: "wooden_figure".to_string(),
        condition: Equals,
        amount: "1".to_string()
      }
    );

    let result = EventRequirement::parse("item wooden_figure >3", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Item {
        id: "wooden_figure".to_string(),
        condition: HigherThan,
        amount: "3".to_string()
      }
    );

    let result = EventRequirement::parse("item wooden_figure <3", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Item {
        id: "wooden_figure".to_string(),
        condition: LessThan,
        amount: "3".to_string()
      }
    );
  }

  // TODO(fran): Figure out how to implement this
  // #[test]
  // fn error_on_missing_item() {
  //   let config = Config::default();
  //   let _result = EventRequirement::parse("item wood 1", &config).unwrap();
  //   // assert_eq!(Err("\"wood\" is not defined as a valid item".to_string()), result);
  //   todo!()
  // }

  #[test]
  fn parses_reputations() {
    let mut config = Config::default();
    config.reputations.push("friends".to_string());

    let result = EventRequirement::parse("reputation friends 1", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Reputation {
        id: "friends".to_string(),
        condition: HigherThan,
        amount: "1".to_string()
      }
    );

    let result = EventRequirement::parse("reputation friends >1", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Reputation {
        id: "friends".to_string(),
        condition: HigherThan,
        amount: "1".to_string()
      }
    );

    let result = EventRequirement::parse("reputation friends <5", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Reputation {
        id: "friends".to_string(),
        condition: LessThan,
        amount: "5".to_string()
      }
    );

    let result = EventRequirement::parse("reputation friends =5", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Reputation {
        id: "friends".to_string(),
        condition: Equals,
        amount: "5".to_string()
      }
    );

    let result = EventRequirement::parse("reputation friends -1", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Reputation {
        id: "friends".to_string(),
        condition: HigherThan,
        amount: "-1".to_string()
      }
    );

    let result = EventRequirement::parse("reputation friends >-1", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Reputation {
        id: "friends".to_string(),
        condition: HigherThan,
        amount: "-1".to_string()
      }
    );

    let result = EventRequirement::parse("reputation friends <-5", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Reputation {
        id: "friends".to_string(),
        condition: LessThan,
        amount: "-5".to_string()
      }
    );

    let result = EventRequirement::parse("reputation friends =-5", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Reputation {
        id: "friends".to_string(),
        condition: Equals,
        amount: "-5".to_string()
      }
    );
  }

  #[test]
  fn error_on_wrong_reputation() {
    let config = Config::default();
    let result = EventRequirement::parse("reputation friends =-5", &config);
    assert_eq!(
      Err("'friends' is not a valid reputation".to_string()),
      result
    );
  }

  #[test]
  fn parses_time_of_day() {
    let config = Config::default();

    let result = EventRequirement::parse("time_of_day morning", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::TimeOfDay {
        id: cuentitos_common::TimeOfDay::Morning,
        condition: Equals
      }
    );

    let result = EventRequirement::parse("time_of_day noon", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::TimeOfDay {
        id: cuentitos_common::TimeOfDay::Noon,
        condition: Equals
      }
    );

    let result = EventRequirement::parse("time_of_day evening", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::TimeOfDay {
        id: cuentitos_common::TimeOfDay::Evening,
        condition: Equals
      }
    );

    let result = EventRequirement::parse("time_of_day night", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::TimeOfDay {
        id: cuentitos_common::TimeOfDay::Night,
        condition: Equals
      }
    );
  }

  #[test]
  fn error_on_wrong_time_of_day() {
    let config = Config::default();
    let result = EventRequirement::parse("time_of_day tonight", &config);
    assert_eq!(
      Err("'tonight' is not a valid time of day".to_string()),
      result
    );
  }

  #[test]
  fn parses_event() {
    let config = Config::default();

    let result = EventRequirement::parse("event an_event", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Event {
        id: "an_event".to_string(),
        condition: Depends
      }
    );

    let result = EventRequirement::parse("event !an_event", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Event {
        id: "an_event".to_string(),
        condition: MutEx
      }
    );
  }

  // TODO(fran): Figure out how to implement this
  // #[test]
  // fn error_on_missing_event() {
  //   let config = Config::default();
  //   let _result = EventRequirement::parse("event that_doesnt_exists", &config).unwrap();
  //   // assert_eq!(Err("\"that_doesnt_exists\" is not a defined event".to_string()), result);
  //   todo!()
  // }

  #[test]
  fn parses_decision() {
    let config = Config::default();

    let result = EventRequirement::parse("decision a_decision", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Decision {
        id: "a_decision".to_string(),
        condition: Depends
      }
    );

    let result = EventRequirement::parse("decision !a_decision", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Decision {
        id: "a_decision".to_string(),
        condition: MutEx
      }
    );
  }

  #[test]
  fn parses_tile() {
    let mut config = Config::default();
    config.tiles.push("forest".to_string());

    let result = EventRequirement::parse("tile forest", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Tile {
        id: "forest".to_string(),
        condition: Depends
      }
    );

    let result = EventRequirement::parse("tile !forest", &config).unwrap();
    assert_eq!(
      result,
      cuentitos_common::EventRequirement::Tile {
        id: "forest".to_string(),
        condition: MutEx
      }
    );
  }

  #[test]
  fn error_on_missing_tile() {
    let config = Config::default();
    let result = EventRequirement::parse("tile forest", &config);
    assert_eq!(Err("'forest' is not a valid tile".to_string()), result);
  }

  #[test]
  fn error_on_wrong_resource_value() {
    let mut config = Config::default();
    config.resources.insert("health".to_string(), Integer);
    let result = EventRequirement::parse("resource health false", &config);
    assert_eq!(
      Err("Invalid value: 'false' for resource 'health'".to_string()),
      result
    );
  }
}
