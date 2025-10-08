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
                Ok((database, warnings)) => {
                    // Print warnings before running
                    for warning in warnings {
                        let file_name = warning
                            .file
                            .as_ref()
                            .and_then(|p| p.file_name())
                            .and_then(|n| n.to_str())
                            .unwrap_or("test.cuentitos");
                        println!(
                            "{}:{}: WARNING: {}",
                            file_name, warning.line, warning.message
                        );
                    }

                    // Run in runtime
                    let mut runtime = cuentitos_runtime::Runtime::new(database);
                    runtime.run();

                    // Process inputs
                    let mut quit_requested = false;
                    if !input_string.is_empty() {
                        for input in input_string.split(',') {
                            let trimmed = input.trim();
                            if trimmed == "q" {
                                quit_requested = true;
                                break;
                            }

                            if !process_input(trimmed, &mut runtime) {
                                break;
                            }

                            // After processing, check if we're waiting for options
                            if runtime.is_waiting_for_option() {
                                render_current_blocks(&runtime);
                                display_options(&runtime);
                            }
                        }
                    }

                    // Final render
                    render_current_blocks(&runtime);

                    // If still waiting for options, display them
                    if runtime.is_waiting_for_option() {
                        display_options(&runtime);
                    }

                    if quit_requested {
                        println!("QUIT");
                    }

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
    let trimmed = input.trim();

    // Check if waiting for option selection
    if runtime.is_waiting_for_option() {
        // Try to parse as option number
        if let Ok(choice) = trimmed.parse::<usize>() {
            // Get option text before selecting (selection clears current_options)
            let option_text = runtime
                .get_current_options()
                .iter()
                .find(|(num, _)| *num == choice)
                .map(|(_, string_id)| runtime.database.strings[*string_id].clone());

            match runtime.select_option(choice) {
                Ok(()) => {
                    // Display the selected option
                    if let Some(text) = option_text {
                        println!("Selected: {}", text);
                    }
                    return true;
                }
                Err(_) => {
                    println!("Invalid option: {}", trimmed);
                    return true; // Continue, will re-display options
                }
            }
        }

        // Handle special commands at option prompt
        match trimmed {
            "q" => return false,
            "n" | "s" => {
                let num_options = runtime.get_current_options().len();
                println!(
                    "Use option numbers (1-{}) to choose (plus q to quit)",
                    num_options
                );
                return true;
            }
            "" => return true, // Ignore empty input
            _ => {
                println!("Invalid option: {}", trimmed);
                return true;
            }
        }
    }

    // Check for GoTo commands
    if trimmed.starts_with("<->") {
        return handle_goto_and_back(trimmed, runtime);
    } else if trimmed.starts_with("->") {
        return handle_goto(trimmed, runtime);
    }

    // Normal (non-option) input processing
    match trimmed {
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
            eprintln!("Unknown command: {}", trimmed);
            false
        }
    }
}

fn handle_goto(input: &str, runtime: &mut cuentitos_runtime::Runtime) -> bool {
    handle_goto_command(input, "-> ", 3, runtime, false)
}

fn handle_goto_and_back(input: &str, runtime: &mut cuentitos_runtime::Runtime) -> bool {
    handle_goto_command(input, "<-> ", 4, runtime, true)
}

/// Common logic for handling goto commands (both -> and <->)
fn handle_goto_command(
    input: &str,
    prefix: &str,
    offset: usize,
    runtime: &mut cuentitos_runtime::Runtime,
    is_call: bool,
) -> bool {
    // Validate syntax: must have space after prefix
    if !input.starts_with(prefix) {
        let arrow = prefix.trim();
        println!(
            "ERROR: Invalid goto command: Expected section name after '{}'",
            arrow
        );
        return true; // Continue waiting for input
    }

    // Parse: extract path after prefix
    let path = &input[offset..];

    // Validate path is not empty
    if path.trim().is_empty() {
        let arrow = prefix.trim();
        println!(
            "ERROR: Invalid goto command: Expected section name after '{}'",
            arrow
        );
        return true;
    }

    // Resolve the path using runtime
    let resolved_path = match runtime.find_section_by_path(path) {
        Ok(resolved) => resolved,
        Err(e) => {
            println!("{}", e);
            return true; // Continue waiting for input
        }
    };

    // Execute the appropriate goto method
    let result = if is_call {
        // For <->, use call-and-return for sections, regular goto for special keywords
        match resolved_path {
            cuentitos_common::ResolvedPath::Section(section_id) => {
                runtime.goto_and_back_section(section_id)
            }
            cuentitos_common::ResolvedPath::Start => runtime.goto_start(),
            cuentitos_common::ResolvedPath::Restart => runtime.goto_restart(),
            cuentitos_common::ResolvedPath::End => runtime.goto_end(),
        }
    } else {
        // For ->, use regular goto for all cases
        match resolved_path {
            cuentitos_common::ResolvedPath::Section(section_id) => runtime.goto_section(section_id),
            cuentitos_common::ResolvedPath::Start => runtime.goto_start(),
            cuentitos_common::ResolvedPath::Restart => runtime.goto_restart(),
            cuentitos_common::ResolvedPath::End => runtime.goto_end(),
        }
    };

    match result {
        Ok(()) => true, // Continue
        Err(e) => {
            println!("{}", e);
            true // Continue waiting for input
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
            cuentitos_common::BlockType::Section(section_id) => {
                // Build the section path by traversing up the hierarchy
                let path = build_section_path(runtime, *section_id);
                println!("-> {}", path);
            }
            cuentitos_common::BlockType::GoTo(_)
            | cuentitos_common::BlockType::GoToAndBack(_)
            | cuentitos_common::BlockType::GoToStart
            | cuentitos_common::BlockType::GoToRestart
            | cuentitos_common::BlockType::GoToEnd => {
                // Goto blocks are not rendered - they're navigation commands
                // that are executed by the runtime
            }
            cuentitos_common::BlockType::Option(_) => {
                // Option blocks are not rendered in normal flow
                // They are displayed via display_options() when waiting for selection
            }
            cuentitos_common::BlockType::End => println!("END"),
        }
    }
}

fn display_options(runtime: &cuentitos_runtime::Runtime) {
    let options = runtime.get_current_options();
    for (num, string_id) in options {
        println!("  {}. {}", num, runtime.database.strings[string_id]);
    }
    println!(">"); // Just print > on its own line
}

fn build_section_path(
    runtime: &cuentitos_runtime::Runtime,
    section_id: cuentitos_common::SectionId,
) -> String {
    // Simply get the path from the section metadata
    let section = &runtime.database.sections[section_id];
    runtime.database.strings[section.path].clone()
}
