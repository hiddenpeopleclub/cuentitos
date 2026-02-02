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

                    // Track what we've rendered to avoid duplicates
                    let mut last_rendered_idx = 0;

                    // Process inputs
                    let mut quit_requested = false;
                    if !input_string.is_empty() {
                        for input in input_string.split(',') {
                            let trimmed = input.trim();
                            let is_variable_command = trimmed == "?";

                            // Auto-step before processing to reach options/content
                            // This allows tests to use "1,s" or "q" instead of "n,1,s" or "n,q"
                            // Only skip auto-step on first input if it's 'n' or 's'
                            let is_option_number = trimmed.parse::<usize>().is_ok();
                            let is_step_or_skip = matches!(trimmed, "n" | "s" | "?");
                            let is_first_input = last_rendered_idx == 0;
                            let need_auto_step =
                                is_option_number || (is_first_input && !is_step_or_skip);
                            let should_auto_step = need_auto_step
                                && !runtime.is_waiting_for_option()
                                && !runtime.has_ended();

                            if should_auto_step {
                                // Keep stepping until we hit options or can't continue
                                while !runtime.is_waiting_for_option()
                                    && !runtime.has_ended()
                                    && runtime.step()
                                {
                                    if report_runtime_error(&mut runtime) {
                                        break;
                                    }
                                    // If we hit options after stepping, break
                                    if runtime.is_waiting_for_option() {
                                        break;
                                    }
                                }

                                // Render new blocks that were stepped over
                                render_path_from(&runtime, last_rendered_idx);
                                last_rendered_idx = runtime.current_path().len();

                                // If we hit options, display them
                                if runtime.is_waiting_for_option() {
                                    display_options(&runtime, false);
                                }
                            }

                            // Check for quit after rendering current state
                            if trimmed == "q" {
                                // If we're at an option prompt, add newline after >
                                if runtime.is_waiting_for_option() {
                                    println!();
                                }
                                quit_requested = true;
                                break;
                            }

                            // Track if we were already at options before processing
                            let was_at_options = runtime.is_waiting_for_option();

                            if is_variable_command {
                                render_path_from(&runtime, last_rendered_idx);
                                last_rendered_idx = runtime.current_path().len();
                                print_variables(&runtime);

                                if runtime.is_waiting_for_option() {
                                    display_options(&runtime, was_at_options);
                                }
                                continue;
                            }

                            if !process_input(trimmed, &mut runtime) {
                                break;
                            }

                            // Render any new blocks after processing input
                            render_path_from(&runtime, last_rendered_idx);
                            last_rendered_idx = runtime.current_path().len();

                            // After processing, check if we're at options
                            // Include parent text only if we were already at options (invalid input case)
                            if runtime.is_waiting_for_option() {
                                display_options(&runtime, was_at_options);
                            }
                        }
                    }

                    // Final render - show any remaining blocks
                    render_path_from(&runtime, last_rendered_idx);

                    // If still waiting for options and we didn't quit, display them
                    if runtime.is_waiting_for_option() && !quit_requested {
                        display_options(&runtime, false);
                    }

                    // Print QUIT only if not at an option prompt
                    if quit_requested && !runtime.is_waiting_for_option() {
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
                    println!("Invalid option: {}", input);
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
                println!("Invalid option: {}", input);
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
                report_runtime_error(runtime);
                true
            } else {
                println!("Cannot continue - reached the end of the script.");
                false
            }
        }
        "s" => {
            if runtime.can_continue() {
                runtime.skip();
                report_runtime_error(runtime);
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

fn report_runtime_error(runtime: &mut cuentitos_runtime::Runtime) -> bool {
    if let Some(err) = runtime.take_last_error() {
        println!("{}", err);
        return true;
    }
    false
}

fn render_path_from(runtime: &cuentitos_runtime::Runtime, start_idx: usize) {
    // Render all blocks from start_idx onwards in current_path
    for &block_id in &runtime.current_path()[start_idx..] {
        let block = &runtime.database.blocks[block_id];
        match block.block_type {
            cuentitos_common::BlockType::Start => println!("START"),
            cuentitos_common::BlockType::String(id) => {
                println!("{}", runtime.database.strings[id])
            }
            cuentitos_common::BlockType::Section(section_id) => {
                // Build the section path
                let path = build_section_path(runtime, section_id);
                println!("-> {}", path);
            }
            cuentitos_common::BlockType::GoTo(_)
            | cuentitos_common::BlockType::GoToAndBack(_)
            | cuentitos_common::BlockType::GoToStart
            | cuentitos_common::BlockType::GoToRestart
            | cuentitos_common::BlockType::GoToEnd
            | cuentitos_common::BlockType::SetVariable { .. }
            | cuentitos_common::BlockType::RequireVariable { .. } => {
                // Goto blocks are not rendered - they're navigation commands
            }
            cuentitos_common::BlockType::Option(_) => {
                // Option blocks are not rendered in normal flow
                // They are displayed via display_options() when waiting for selection
            }
            cuentitos_common::BlockType::End => println!("END"),
        }
    }
}

fn print_variables(runtime: &cuentitos_runtime::Runtime) {
    for (name, value) in runtime.list_variables() {
        println!("{}: {}", name, value.to_display_string());
    }
}

fn display_options(runtime: &cuentitos_runtime::Runtime, include_parent: bool) {
    let options = runtime.get_current_options();

    // Display the parent block text if requested (e.g., when redisplaying after error)
    if include_parent {
        let option_block_ids = runtime.get_current_option_block_ids();
        if let Some(&first_option_block_id) = option_block_ids.first() {
            if let Some(parent_id) = runtime.database.blocks[first_option_block_id].parent_id {
                let parent_block = &runtime.database.blocks[parent_id];
                if let cuentitos_common::BlockType::String(string_id) = parent_block.block_type {
                    println!("{}", runtime.database.strings[string_id]);
                }
            }
        }
    }

    for (num, string_id) in options {
        println!("  {}. {}", num, runtime.database.strings[string_id]);
    }
    print!("> "); // Print > with space, responses continue on same line
    std::io::Write::flush(&mut std::io::stdout()).ok();
}

fn build_section_path(
    runtime: &cuentitos_runtime::Runtime,
    section_id: cuentitos_common::SectionId,
) -> String {
    // Simply get the path from the section metadata
    let section = &runtime.database.sections[section_id];
    runtime.database.strings[section.path].clone()
}
