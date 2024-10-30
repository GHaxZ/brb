/*  TODO:
*   Clean up the code
*/

use anyhow::Result;

mod args;
mod chat;
mod config;
mod song;
mod state;

fn main() -> Result<()> {
    // Parse arguments
    args::parse()
}
