use godot::private::class_macros::property::PropertyHintInfo;
use std::path::PathBuf;
use cuentitos_runtime::Runtime;
use std::str::FromStr;
use godot::engine::*;
use godot::prelude::*;
use cuentitos_common::*;
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

#[derive(GodotClass)]
#[class(tool, init, base=Resource)]
struct CuentitosDatabase {
    database: Database,
    base: Base<Resource>,
}

impl CuentitosDatabase {
  fn new(source: String) -> Gd<Self> {
    let database = cuentitos_compiler::compile_database(&source).unwrap();
    Gd::from_init_fn(|base| {
      Self {
        database,
        base,
      }
    })
  }
}

#[derive(GodotClass)]
#[class(tool, init, base=Object)]
struct Cuentitos {
  config: Config,
  runtime: Runtime,
  base: Base<Object>
}

#[godot_api]
impl Cuentitos {
  #[func]
  fn compile(&mut self, source: String) -> Gd<CuentitosDatabase> {
    CuentitosDatabase::new(source)
  }

  #[func]
  fn load(&mut self, script: String) {
    let database = load::<CuentitosDatabase>(script);
  }

}  
//   #[func]
//   fn load_config(&mut self, content: String) {
//     self.config = Config::from_str(&content).unwrap();
//   }



//   }
  
//   // pub fn reset_story(&mut self)  {}
//   // pub fn reset_state(&mut self)  {}
//   // pub fn reset_all(&mut self)  {}

//   // pub fn set_locale<T>(&mut self, locale: T) -> Result<(), String> {}

//   // pub fn set_seed(&mut self, seed: u64) {}
//   // pub fn divert(&mut self, section: &Section) -> Result<Vec<Block>, RuntimeError>  {}

//   // pub fn boomerang_divert(&mut self, section: &Section) -> Result<Vec<Block>, RuntimeError>  {}

//   // pub fn peek_next(&self) -> Result<Output, RuntimeError>  {}

//   // pub fn next_block(&mut self) -> Result<Output, RuntimeError>  {}

//   // pub fn progress_story(&mut self) -> Result<Output, RuntimeError> {}

//   // pub fn skip(&mut self) -> Result<Output, RuntimeError>  {}
//   // pub fn skip_all(&mut self) -> Result<Output, RuntimeError>  {}

//   // pub fn get_block(&self, stack_data: &BlockStackData) -> Result<Block, RuntimeError>  {}

//   // pub fn current(&self) -> Result<Output, RuntimeError>  {}

//   // pub fn pick_choice(&mut self, choice: usize) -> Result<Output, RuntimeError>  {}

//   // pub fn set_variable<R, T>(&mut self, variable: R, value: T) -> Result<(), RuntimeError> {}

//   // pub fn get_variable_kind<R>(&self, variable: R) -> Result<VariableKind, RuntimeError> {}

//   // pub fn get_variable<R, T>(&self, variable: R) -> Result<T, RuntimeError> {}

//   // pub fn apply_modifier(&mut self, modifier: &Modifier) -> Result<(), RuntimeError>  {}

//   // pub fn get_current_choices_strings(&self) -> Result<Vec<String>, RuntimeError>  {}
// }

