/*  TODO:
*   Clean up the code
*   Add a way to execute commands when starting and ending
*/
use std::io;

mod args;
mod chat;
mod config;
mod state;

fn main() -> io::Result<()> {
    // Parse arguments
    args::parse()
}
