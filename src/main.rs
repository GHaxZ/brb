/*  TODO:
*   Clean up the code
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
