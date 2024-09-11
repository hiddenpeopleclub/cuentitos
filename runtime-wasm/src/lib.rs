use cuentitos_runtime::Runtime;
use cuentitos_runtime::Database;
use js_sys::Uint8Array;

mod utils;

use wasm_bindgen::prelude::*;

static mut RUNTIMES: Vec<Runtime> = vec![];


#[wasm_bindgen]
extern "C" {
  #[wasm_bindgen(js_namespace = console)]
  fn log(s: &str);
}


#[wasm_bindgen(start)]
fn main() -> Result<(), JsValue> {
    utils::set_panic_hook();
    Ok(())
}

#[wasm_bindgen]
pub fn load(database: Uint8Array) -> usize {
  
  let database = Database::from_u8(database.to_vec().as_slice()).unwrap();
  let idx;
  unsafe {
    idx = RUNTIMES.len();
    RUNTIMES.push(Runtime::new(database.clone()));
  }
  idx
}

#[wasm_bindgen]
pub fn progress_story(id: usize) -> String
{
  unsafe {
    match RUNTIMES[id].progress_story() {
        Ok(output) => {
          serde_json::to_string(&output).unwrap()
        }
        Err(err) => {
          panic!("Error: {}", err)
        }
    }    
  }
}

#[wasm_bindgen]
pub fn skip(id: usize) -> String {
  unsafe {
    match RUNTIMES[id].skip() {
      Ok(output) => {
        serde_json::to_string(&output).unwrap()
      }
      Err(err) => {
        panic!("Error: {}", err)
      }
    }
  }        
}


#[wasm_bindgen]
pub fn pick_choice(id: usize, choice: usize) -> String {
  unsafe {
    match RUNTIMES[id].pick_choice(choice) {
        Ok(output) => {
          serde_json::to_string(&output).unwrap()
        }
        Err(err) => {
          panic!("Error: {}", err)
        }
    }    
  }
}

// pub fn reset_story(id: usize) {
//   unsafe {
//     RUNTIMES[id].reset_story();
//   }
// }
// pub fn reset_state(&mut self) {}
// pub fn reset_all(&mut self) {}
// pub fn set_seed(&mut self, seed: u64) {}
// pub fn divert(&mut self, section: &Section) -> String { "".to_string() }
// pub fn boomerang_divert(&mut self, section: &Section) -> String { "".to_string() }

// pub fn peek_next(&self) -> String { "".to_string() }

// pub fn next_block(&mut self) -> String { "".to_string() }

// pub fn skip_all(&mut self) -> String { "".to_string() }

// pub fn current(&self) -> String { "".to_string() }

// pub fn set_variable<R, T>(&mut self, variable: R, value: T) -> Result<(), RuntimeError>
// where
//   T: Display + std::str::FromStr + Default,
//   R: AsRef<str>,
// {
//   let variable = variable.as_ref().to_string();
//   if self.database.config.variables.contains_key(&variable) {
//     let t = std::any::type_name::<T>();
//     if (t == "i32" && self.database.config.variables[&variable] == VariableKind::Integer)
//       || (t == "f32" && self.database.config.variables[&variable] == VariableKind::Float)
//       || (t == "bool" && self.database.config.variables[&variable] == VariableKind::Bool)
//       || (t == "alloc::string::String"
//         && self.database.config.variables[&variable] == VariableKind::String)
//       || (t == "&str" && self.database.config.variables[&variable] == VariableKind::String)
//       || self.is_valid_enum::<T>(&value.to_string())
//     {
//       self
//         .game_state
//         .variables
//         .insert(variable, value.to_string());
//     } else {
//       return Err(RuntimeError::UnsupportedVariableType {
//         type_found: t.to_string(),
//       });
//     }
//   } else {
//     return Err(RuntimeError::VariableDoesntExist(variable));
//   }
//   Ok(())
// }

// pub fn get_variable_kind<R>(&self, variable: R) -> Result<VariableKind, RuntimeError>
// where
//   R: AsRef<str>,
// {
//   let variable = variable.as_ref();

//   if self.database.config.variables.contains_key(variable) {
//     Ok(self.database.config.variables[variable].clone())
//   } else {
//     Err(RuntimeError::VariableDoesntExist(variable.to_string()))
//   }
// }

// pub fn get_variable<R, T>(&self, variable: R) -> Result<T, RuntimeError>
// where
//   T: Display + std::str::FromStr + Default,
//   R: AsRef<str>,
// {
//   let variable = variable.as_ref().to_string();
//   let value = match self.game_state.variables.get(&variable) {
//     Some(value) => value.clone(),
//     None => T::default().to_string(),
//   };

//   if self.database.config.variables.contains_key(&variable) {
//     let t = std::any::type_name::<T>();
//     if (t == "i32" && self.database.config.variables[&variable] == VariableKind::Integer)
//       || (t == "f32" && self.database.config.variables[&variable] == VariableKind::Float)
//       || (t == "bool" && self.database.config.variables[&variable] == VariableKind::Bool)
//       || t == "alloc::string::String"
//       || t == "&str"
//       || self.is_valid_enum::<T>(&value)
//     {
//       match value.parse::<T>() {
//         Ok(value) => Ok(value),
//         Err(_) => Err(RuntimeError::UnknownParsingError),
//       }
//     } else {
//       Err(RuntimeError::UnsupportedVariableType {
//         type_found: t.to_string(),
//       })
//     }
//   } else {
//     Err(RuntimeError::VariableDoesntExist(variable))
//   }
// }

// pub fn apply_modifier(&mut self, modifier: &Modifier) -> Result<(), RuntimeError> {
//   match self.get_variable_kind(&modifier.variable)? {
//     VariableKind::Integer => {
//       let value = &modifier.value.parse::<i32>();
//       match value {
//         Ok(value) => self.apply_integer_modifier(&modifier.variable, *value, &modifier.operator),
//         Err(e) => Err(RuntimeError::ParseIntError(e.clone())),
//       }
//     }
//     VariableKind::Float => {
//       let value = &modifier.value.parse::<f32>();
//       match value {
//         Ok(value) => self.apply_float_modifier(&modifier.variable, *value, &modifier.operator),
//         Err(e) => Err(RuntimeError::ParseFloatError(e.clone())),
//       }
//     }
//     VariableKind::Bool => {
//       let value = &modifier.value.parse::<bool>();
//       match value {
//         Ok(value) => self.set_variable(&modifier.variable, *value),
//         Err(e) => Err(RuntimeError::ParseBoolError(e.clone())),
//       }
//     }
//     _ => self.set_variable(&modifier.variable, modifier.value.clone()),
//   }
// }

// pub fn get_current_choices_strings(&self) -> Result<Vec<String>, RuntimeError> {
//   let mut choices_strings = Vec::default();
//   for choice in &self.choices {
//     if let cuentitos_common::Block::Choice { id, settings: _ } =
//       self.get_cuentitos_block(*choice)?
//     {
//       choices_strings.push(self.database.i18n.get_translation(&self.current_locale, id));
//     }
//   }

//   Ok(choices_strings)
// }
