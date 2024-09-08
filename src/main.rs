//  TODO: Display chat aswell (twitch-irc crate)

use std::io;

mod args;
mod config;
mod state;

fn main() -> io::Result<()> {
    args::parse()
}
