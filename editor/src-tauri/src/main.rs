#![cfg_attr(
  all(not(debug_assertions), target_os = "windows"),
  windows_subsystem = "windows"
)]

use cuentitos_common::Database;
use cuentitos_common::EventId;
use cuentitos_common::Item;
use cuentitos_runtime::Runtime;
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::str::FromStr;
use std::sync::Mutex;
use tauri::Manager;
use tauri::State;

#[tauri::command]
fn next_event(state: State<'_, EditorState>) -> Option<cuentitos_runtime::Event> {
  state.runtime.lock().unwrap().next_event()
}

#[tauri::command]
fn get_event_list(
  state: State<'_, EditorState>,
) -> HashMap<String, Result<cuentitos_common::Event, String>> {
  state.parser.lock().unwrap().events.clone()
}

#[tauri::command]
fn load_event(
  state: State<'_, EditorState>,
  event_id: EventId,
) -> Option<cuentitos_runtime::Event> {
  state.runtime.lock().unwrap().load_event(event_id)
}

#[tauri::command]
fn set_choice(
  state: State<'_, EditorState>,
  choice_id: usize,
) -> Result<cuentitos_runtime::EventResult, String> {
  state.runtime.lock().unwrap().set_choice(choice_id)
}

#[tauri::command]
fn get_config(state: State<'_, EditorState>) -> cuentitos_common::Config {
  state.runtime.lock().unwrap().database.config.clone()
}

#[tauri::command]
fn get_items(state: State<'_, EditorState>) -> Vec<Item> {
  state.runtime.lock().unwrap().database.items.clone()
}

#[tauri::command]
fn set_locale(state: State<'_, EditorState>, locale: String) -> Result<(), String> {
  state.runtime.lock().unwrap().set_locale(locale)
}

#[tauri::command]
fn set_tile(state: State<'_, EditorState>, tile: String) -> Result<(), String> {
  state.runtime.lock().unwrap().set_tile(tile)
}

#[tauri::command]
fn set_resource_bool(
  state: State<'_, EditorState>,
  resource: String,
  value: bool,
) -> Result<(), String> {
  state.runtime.lock().unwrap().set_resource(resource, value)
}

#[tauri::command]
fn set_resource_int(
  state: State<'_, EditorState>,
  resource: String,
  value: i32,
) -> Result<(), String> {
  state.runtime.lock().unwrap().set_resource(resource, value)
}

#[tauri::command]
fn set_resource_float(
  state: State<'_, EditorState>,
  resource: String,
  value: f32,
) -> Result<(), String> {
  state.runtime.lock().unwrap().set_resource(resource, value)
}

#[tauri::command]
fn set_item(state: State<'_, EditorState>, item: String, count: u8) -> Result<(), String> {
  state.runtime.lock().unwrap().set_item(item, count)
}

#[tauri::command]
fn reload_db(state: State<'_, EditorState>) -> Result<(), String> {
  if let Ok(parser) = cuentitos_compiler::compile(&state.source_path, &state.destination_path) {
    let mut f = File::open(&state.destination_path).expect("no file found");
    let metadata = std::fs::metadata(&state.destination_path).expect("unable to read metadata");
    let mut buffer = vec![0; metadata.len() as usize];
    f.read_exact(&mut buffer).expect("buffer overflow");
    let database = Database::from_u8(&buffer).unwrap();

    let mut p = state.parser.lock().unwrap();

    p.events = parser.events;
    p.items = parser.items;
    p.config = parser.config;
    p.i18n = parser.i18n;

    state.runtime.lock().unwrap().database = database;
    Ok(())
  } else {
    Err("Compilation Failed".to_string())
  }
}

#[derive(Debug, Default)]
struct EditorState {
  source_path: PathBuf,
  destination_path: PathBuf,
  parser: Mutex<cuentitos_compiler::parser::Parser>,
  runtime: Mutex<Runtime>,
}

fn main() {
  tauri::Builder::default()
    .setup(|app| {
      match app.get_cli_matches() {
        // `matches` here is a Struct with { args, subcommand }.
        // `args` is `HashMap<String, ArgData>` where `ArgData` is a struct with { value, occurrences }.
        // `subcommand` is `Option<Box<SubcommandMatches>>` where `SubcommandMatches` is a struct with { name, matches }.
        Ok(matches) => {
          let source = matches.args["source"].value.as_str().unwrap_or(".");
          let destination = matches.args["destination"]
            .value
            .as_str()
            .unwrap_or("./build");
          let db = matches.args["db"].value.as_str().unwrap_or("cuentitos.db");

          let source = PathBuf::from_str(source)?;
          let mut destination = PathBuf::from_str(destination)?;
          destination.push(db);

          let parser = cuentitos_compiler::compile(&source, &destination)?;

          let mut f = File::open(&destination).expect("no file found");
          let metadata = std::fs::metadata(&destination).expect("unable to read metadata");
          let mut buffer = vec![0; metadata.len() as usize];
          f.read_exact(&mut buffer).expect("buffer overflow");
          let database = Database::from_u8(&buffer).unwrap();

          let runtime = Runtime::new(database);

          app.manage(EditorState {
            source_path: source,
            destination_path: destination,
            parser: parser.into(),
            runtime: runtime.into(),
          });
        }
        Err(_) => {}
      }

      Ok(())
    })
    .invoke_handler(tauri::generate_handler![
      reload_db,
      next_event,
      get_event_list,
      load_event,
      set_choice,
      get_config,
      get_items,
      set_locale,
      set_tile,
      set_resource_bool,
      set_resource_float,
      set_resource_int,
      set_item
    ])
    .run(tauri::generate_context!())
    .expect("error while running tauri application");
}
