//  TODO: Display chat aswell (twitch-irc crate)
//  TODO: Progress bar (Gauge) for time progress
//  TODO: Allow configuration where twitch name and other configs are stored

use std::io;

mod args;
mod state;

fn main() -> io::Result<()> {
    args::parse()
}
