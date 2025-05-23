use std::env;
use std::io::{self, Write};
use std::path::Path;
use std::process::{Command, Stdio, Child}; // [cite: 2]

fn main() {
    loop {
        print!("my_shell> ");
        io::stdout().flush().unwrap_or_else(|e| { // [cite: 2]
            eprintln!("Error flushing stdout: {}", e);
        });

        let mut input = String::new(); // [cite: 2]
        match io::stdin().read_line(&mut input) { // [cite: 2]
            Ok(0) => {
                // EOF (Ctrl+D)
                println!("\nExiting my_shell.");
                break;
            }
            Ok(_) => {
                let trimmed_input = input.trim();

                if trimmed_input.is_empty() {
                    continue;
                }

                if trimmed_input == "exit" {
                    println!("Exiting my_shell.");
                    break;
                }

                // Handle commands, including potential pipes
                execute_pipeline(trimmed_input);
            }
            Err(error) => {
                eprintln!("Error reading input: {}", error);
                // Optionally break or decide to continue
                // break;
            }
        }
    }
}

fn execute_pipeline(line: &str) {
    let commands_str: Vec<&str> = line.split('|').map(|s| s.trim()).collect();
    let num_commands = commands_str.len();
    let mut children: Vec<Child> = Vec::new(); // [cite: 2]
    let mut previous_stdout: Option<Stdio> = None;

    for (i, command_segment) in commands_str.iter().enumerate() {
        if command_segment.is_empty() {
            eprintln!("Error: empty command in pipeline.");
            // Clean up any spawned children if an error occurs mid-pipeline
            for mut child in children {
                let _ = child.kill(); // Attempt to kill, ignore error if already exited
                let _ = child.wait(); // Wait to clean up resources
            }
            return;
        }

        let mut parts = command_segment.split_whitespace();
        let command_name = match parts.next() {
            Some(cmd) => cmd,
            None => { // Should be caught by empty check above, but good for safety
                eprintln!("Error: empty command segment.");
                for mut child in children {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                return;
            }
        };
        let args: Vec<&str> = parts.collect();

        // Handle built-in 'cd' command separately as it affects the parent process
        if command_name == "cd" {
            if num_commands > 1 {
                eprintln!("'cd' cannot be part of a pipeline.");
                 for mut child in children {
                    let _ = child.kill();
                    let _ = child.wait();
                }
                return;
            }
            let new_dir = args.get(0).map_or_else(
                || env::var("HOME").unwrap_or_else(|_| "/".to_string()), // Default to HOME or root
                |x| x.to_string()
            );
            let root = Path::new(&new_dir);
            if let Err(e) = env::set_current_dir(&root) {
                eprintln!("Error changing directory to {}: {}", new_dir, e);
            }
            return; // 'cd' is done, no external command to run
        }

        let mut current_command = Command::new(command_name); // [cite: 2]
        current_command.args(&args);

        // Setup stdin for the current command
        if let Some(prev_stdout_handle) = previous_stdout.take() {
            current_command.stdin(prev_stdout_handle);
        }

        // Setup stdout for the current command
        if i < num_commands - 1 { // If it's not the last command in the pipe
            current_command.stdout(Stdio::piped());
        } else { // Last command or only command
            current_command.stdout(Stdio::inherit()); // Output to terminal
        }

        // Spawn the command
        match current_command.spawn() {
            Ok(mut child) => {
                if i < num_commands - 1 { // If there's a next command, pass stdout
                    previous_stdout = child.stdout.take().map(Stdio::from);
                }
                children.push(child);
            }
            Err(e) => {
                eprintln!("Error executing command '{}': {}", command_name, e);
                // Clean up any previously spawned children in the pipeline
                for mut prev_child in children {
                   let _ = prev_child.kill();
                   let _ = prev_child.wait();
                }
                return; // Stop processing this pipeline
            }
        }
    }

    // Wait for all children in the pipeline to complete
    for (i, mut child) in children.into_iter().enumerate() {
        match child.wait() {
            Ok(status) => {
                if !status.success() {
                    // Get the command name for better error reporting
                    let command_segment = commands_str[i];
                    let command_name = command_segment.split_whitespace().next().unwrap_or("Unknown command");
                    eprintln!("Command '{}' failed with status: {}", command_name, status);
                }
            }
            Err(e) => {
                let command_segment = commands_str[i];
                let command_name = command_segment.split_whitespace().next().unwrap_or("Unknown command");
                eprintln!("Error waiting for command '{}': {}", command_name, e);
            }
        }
    }
}