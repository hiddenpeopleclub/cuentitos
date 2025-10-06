use clap::{Parser, Subcommand};
use cuentitos_parser::Parser as CuentitosParser;
use std::path::PathBuf;
/// Cuentitos - A narrative game engine with probability at its core

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Run a Cuentitos script
    Run {
        /// Path to the script file to run
        script_path: PathBuf,
        /// Comma-separated list of inputs (e.g., "n,n,s,q")
        input_string: String,
    },
}

fn main() {
    let cli = Args::parse();

    match cli.command {
        Commands::Run {
            script_path,
            input_string,
        } => {
            // Read the script file
            let script = match std::fs::read_to_string(&script_path) {
                Ok(content) => content,
                Err(err) => {
                    eprintln!("Error reading script file: {}", err);
                    std::process::exit(1);
                }
            };

            // Create parser with file path
            let mut parser = CuentitosParser::with_file(script_path);

            // Parse it
            match parser.parse(&script) {
                Ok(database) => {
                    // Run in runtime
                    let mut runtime = cuentitos_runtime::Runtime::new(database);
                    runtime.run();

                    // Process inputs
                    if !input_string.is_empty() {
                        for input in input_string.split(',') {
                            if !process_input(input.trim(), &mut runtime) {
                                break;
                            }
                        }
                    }

                    // Final render
                    render_current_blocks(&runtime);

                    if runtime.has_ended() {
                        runtime.stop();
                    } else {
                        eprintln!("\nWarning: Script did not reach the End block.");
                    }
                }
                Err(err) => {
                    println!("{}", err);
                    std::process::exit(1);
                }
            }
        }
    }
}

fn process_input(input: &str, runtime: &mut cuentitos_runtime::Runtime) -> bool {
    match input {
        "n" => {
            if runtime.can_continue() {
                runtime.step();
                true
            } else {
                println!("Cannot continue - reached the end of the script.");
                false
            }
        }
        "s" => {
            if runtime.can_continue() {
                runtime.skip();
                true
            } else {
                println!("Cannot skip - reached the end of the script.");
                false
            }
        }
        "q" => false,
        "" => true, // Ignore empty input
        _ => {
            eprintln!("Unknown command: {}", input);
            false
        }
    }
}

fn render_current_blocks(runtime: &cuentitos_runtime::Runtime) {
    for block in runtime.current_blocks() {
        match &block.block_type {
            cuentitos_common::BlockType::Start => println!("START"),
            cuentitos_common::BlockType::String(id) => {
                println!("{}", runtime.database.strings[*id])
            }
            cuentitos_common::BlockType::Section {
                id: _,
                display_name: _,
            } => {
                // Build the section path by traversing up the hierarchy
                let path = build_section_path(runtime, &block);
                println!("-> {}", path);
            }
            cuentitos_common::BlockType::GoToSection { .. } => {
                // GoToSection blocks are not rendered - they're navigation commands
                // that are executed by the runtime
            }
            cuentitos_common::BlockType::End => println!("END"),
        }
    }
}

fn build_section_path(
    runtime: &cuentitos_runtime::Runtime,
    current_block: &cuentitos_common::Block,
) -> String {
    let mut path_parts = Vec::new();

    // Add current section's display name
    if let cuentitos_common::BlockType::Section { display_name, .. } = &current_block.block_type {
        path_parts.push(display_name.clone());
    }

    // Traverse up the hierarchy to find parent sections
    let mut current_id = current_block.parent_id;
    while let Some(parent_id) = current_id {
        let parent_block = &runtime.database.blocks[parent_id];
        if let cuentitos_common::BlockType::Section { display_name, .. } = &parent_block.block_type
        {
            path_parts.insert(0, display_name.clone());
        }
        current_id = parent_block.parent_id;
    }

    // Join with " \ " as separator (backslash with spaces)
    path_parts.join(" \\ ")
}
