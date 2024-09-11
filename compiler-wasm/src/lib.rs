use serde::ser::Serialize;
mod utils;

use wasm_bindgen::prelude::*;

#[wasm_bindgen]
pub fn compile(script: &str, config: &str) -> String {
  match cuentitos_compiler::compile_from_str(script, config) {
    Ok(db) => {
      let mut buf: Vec<u8> = Vec::new();
      let mut serializer = rmp_serde::Serializer::new(&mut buf);

      db.serialize(&mut serializer).unwrap();

      let result = String::from_utf8(buf).unwrap();
      return result
    }
    Err(e) => {
      return format!("Error: {}", e)
    }
  }
}
