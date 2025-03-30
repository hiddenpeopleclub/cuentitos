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
    let mut current_sections: Vec<cuentitos_common::Block> = Vec::new();
    let mut last_shown_sections: Vec<cuentitos_common::Block> = Vec::new();

    for block in runtime.current_blocks() {
        match block.block_type {
            cuentitos_common::BlockType::Start => println!("START"),
            cuentitos_common::BlockType::String(id) => {
                // Show the current section path before each text block if it has changed
                if current_sections != last_shown_sections {
                    if !current_sections.is_empty() {
                        let path = current_sections
                            .iter()
                            .filter_map(|section| {
                                if let cuentitos_common::BlockType::Section(id) = section.block_type
                                {
                                    Some(runtime.database.strings[id].clone())
                                } else {
                                    None
                                }
                            })
                            .collect::<Vec<_>>();

                        println!("-> {}", path.join(" \\ "));
                    }
                    last_shown_sections = current_sections.clone();
                }
                println!("{}", runtime.database.strings[id]);
            }
            cuentitos_common::BlockType::End => println!("END"),
            cuentitos_common::BlockType::Section(_) => {
                // Update current sections based on level
                while let Some(last) = current_sections.last() {
                    if last.level >= block.level {
                        current_sections.pop();
                    } else {
                        break;
                    }
                }
                current_sections.push(block.clone());
            }
        }
    }
}
