/*  TODO:
*   Clean up the code
*   Improve error handling with anyhow
*   Fix the program taking longer to exit, waiting for exit commands to execute
*/
use std::io;

mod args;
mod chat;
mod config;
mod song;
mod state;

fn main() -> io::Result<()> {
    // Parse arguments
    args::parse()
}
