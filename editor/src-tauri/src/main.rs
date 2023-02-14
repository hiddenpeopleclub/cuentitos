#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

use std::sync::Mutex;
use tauri::State;
use cuentitos_common::Database;
use std::io::Read;
use std::fs::File;
use tauri::Manager;
use std::str::FromStr;
use std::path::PathBuf;
use cuentitos_runtime::Runtime;

// Learn more about Tauri commands at https://tauri.app/v1/guides/features/command
#[tauri::command]
fn greet(name: &str) -> String {
    format!("Hello, {}! You've been greeted from Rust!", name)
}

#[tauri::command]
fn next_event(state: State<'_, EditorState>) -> cuentitos_runtime::Event {
  state.runtime.lock().unwrap().next_event().unwrap()
}

#[derive(Debug, Default)]
struct EditorState {
  parser: cuentitos_compiler::parser::Parser,
  runtime: Mutex<Runtime>
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
                  let destination = matches.args["destination"].value.as_str().unwrap_or("./build");
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
                    parser,
                    runtime: runtime.into()
                  });
                  // if let (String(source), String(destination), String(database)) = (matches.args["source"].value, matches.args["source"].value, matches.args["source"].value) {
                  //   println!("HERE")  
                  // }
                  
                }
                Err(_) => {}
              }
                          
              Ok(())                    
        })
        .invoke_handler(tauri::generate_handler![greet, next_event])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
