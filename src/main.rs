use std::io::{self, Write}; // Import Write for flushing stdout

fn main() {
    loop {
        print!("my_shell> "); // Display a prompt
        io::stdout().flush().unwrap(); // Make sure the prompt is displayed immediately

        let mut input = String::new(); // Create a mutable string to store input [cite: 2]
        match io::stdin().read_line(&mut input) { // Read a line from stdin [cite: 2]
            Ok(_) => {
                // Input successfully read
                let trimmed_input = input.trim(); // Remove trailing newline

                if trimmed_input == "exit" {
                    println!("Exiting my_shell.");
                    break; // Exit the loop if the user types "exit"
                }

                if !trimmed_input.is_empty() {
                    // TODO: Parse and execute the command
                    println!("You entered: {}", trimmed_input);
                }
            }
            Err(error) => {
                eprintln!("Error reading input: {}", error);
                // Decide if you want to break or continue on error
            }
        }
    }
}