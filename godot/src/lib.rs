use cuentitos_runtime::Database;


use rmp_serde::Serializer;
use serde::Serialize;
use cuentitos_runtime::Runtime;
use godot::engine::*;
use godot::prelude::*;
use godot::obj::Gd;

struct CuentitosExtension;

#[gdextension]
unsafe impl ExtensionLibrary for CuentitosExtension {
  fn on_level_init(level: InitLevel) {
    if level == InitLevel::Scene {
      // The StringName identifies your singleton and can be
      // used later to access it.
      Engine::singleton().register_singleton(
        StringName::from("Cuentitos"),
        Cuentitos::new_alloc().upcast(),
      );
    }
  }

  fn on_level_deinit(level: InitLevel) {
    if level == InitLevel::Scene {
      // Unregistering is needed to avoid memory leaks and 
      // warnings, especially for hot reloading.
      Engine::singleton().unregister_singleton(
          StringName::from("Cuentitos")
      );
    }
  }
}

// -------
// RUNTIME OUTPUTS
// -------
#[derive(GodotClass)]
#[class(init, base=Object)]
struct Output {
  #[var]
  text: GString,
  #[var]
  choices: Array<GString>,
  #[var]
  blocks: GString,
  base: Base<Object>
}

impl Output {
  fn new_gd(cuentitos_output: cuentitos_runtime::Output) -> Gd<Self> {
    let text = cuentitos_output.text.into();
    let mut choices = Array::new();
    let blocks = serde_json::to_string(&cuentitos_output.blocks).unwrap().into();
    
    for choice in cuentitos_output.choices {
      choices.push(choice.into())
    }

    Gd::from_init_fn(|base| {
      Self {
        text,
        choices,
        blocks,
        base,
      }
    })
  }
}




#[derive(GodotClass)]
#[class(tool, init, base=Object)]
struct Cuentitos {
  runtime: Runtime,
  base: Base<Object>
}

#[godot_api]
impl Cuentitos {
  #[func]
  fn compile(&mut self, source: String) -> PackedByteArray
  {
    let database = cuentitos_compiler::compile_database(&source).unwrap();

    let mut buf: Vec<u8> = Vec::new();
    let mut serializer = Serializer::new(&mut buf);

    database.serialize(&mut serializer).unwrap();

    PackedByteArray::from(buf.as_slice())
  }

  #[func]
  fn load(&mut self, database: PackedByteArray) {
    let database = Database::from_u8(database.as_slice()).unwrap();
    self.runtime = Runtime::new(database.clone());
  }

  #[func]
  fn progress_story(&mut self) -> Gd<Output>
  {
    match self.runtime.progress_story() {
        Ok(output) => {
          Output::new_gd(output)
        }
        Err(err) => {
          panic!("Error: {}", err)
        }
    }
  }

  #[func]
  fn skip(&mut self) -> Gd<Output> {
    match self.runtime.skip() {
        Ok(output) => {
          Output::new_gd(output)
        }
        Err(err) => {
          panic!("Error: {}", err)
        }
    }        
  }

  #[func]
  fn current(&self) -> Gd<Output> {
    match self.runtime.current() {
        Ok(output) => {
          Output::new_gd(output)
        }
        Err(err) => {
          panic!("Error: {}", err)
        }
    }            
  }

  #[func]
  fn pick_choice(&mut self, choice: u32) -> Gd<Output> {
    match self.runtime.pick_choice(choice.try_into().unwrap()) {
        Ok(output) => {
          Output::new_gd(output)
        }
        Err(err) => {
          panic!("Error: {}", err)
        }
    }    
  }
}  
//   // pub fn reset_story(&mut self)  {}
//   // pub fn reset_state(&mut self)  {}
//   // pub fn reset_all(&mut self)  {}

//   // pub fn set_locale<T>(&mut self, locale: T) -> Result<(), String> {}

//   // pub fn set_seed(&mut self, seed: u64) {}
//   // pub fn divert(&mut self, section: &Section) -> Result<Vec<Block>, RuntimeError>  {}

//   // pub fn boomerang_divert(&mut self, section: &Section) -> Result<Vec<Block>, RuntimeError>  {}

//   // pub fn peek_next(&self) -> Result<Output, RuntimeError>  {}


//   // pub fn skip_all(&mut self) -> Result<Output, RuntimeError>  {}

//   // pub fn get_block(&self, stack_data: &BlockStackData) -> Result<Block, RuntimeError>  {}

//   // pub fn current(&self) -> Result<Output, RuntimeError>  {}


//   // pub fn set_variable<R, T>(&mut self, variable: R, value: T) -> Result<(), RuntimeError> {}

//   // pub fn get_variable_kind<R>(&self, variable: R) -> Result<VariableKind, RuntimeError> {}

//   // pub fn get_variable<R, T>(&self, variable: R) -> Result<T, RuntimeError> {}

//   // pub fn apply_modifier(&mut self, modifier: &Modifier) -> Result<(), RuntimeError>  {}

//   // pub fn get_current_choices_strings(&self) -> Result<Vec<String>, RuntimeError>  {}
// }

