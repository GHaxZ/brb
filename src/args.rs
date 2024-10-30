use anyhow::{Context, Result};
use std::{io, time::Duration};

use crate::{
    config::{Config, TomlColor},
    state::App,
};
use clap::{Arg, ArgAction, ArgGroup, Command};

// A time unit
#[derive(Clone)]
enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
}

// A value which is of a specific time unit
#[derive(Clone)]
struct TimeValue {
    value: u64,
    unit: TimeUnit,
}

// Parse CLI arguments
pub fn parse() -> Result<()> {
    // Create the app and load the config file
    let mut app = App::default();
    let mut config = Config::load()?;

    let matches = command(&config).get_matches();

    // Check CLI arguments and update the config if necessary

    if matches.get_flag("dir") {
        return output_dir();
    }

    if let Some(text) = matches.get_one::<String>("text") {
        config.set_text(text.clone());
    }

    if let Some(&chat) = matches.get_one::<bool>("chat") {
        config.set_chat(chat);
    }

    if let Some(&song_display) = matches.get_one::<bool>("song-display") {
        config.set_song_display(song_display);
    }

    if let Some(twitch) = matches.get_one::<String>("twitch") {
        config.set_twitch_channel(twitch.clone());
    }

    if let Some(color) = matches.get_one::<TomlColor>("color") {
        config.set_color(color.clone());
    }

    if let Some(&hide_timer) = matches.get_one::<bool>("hide-timer") {
        config.set_hide_timer(hide_timer);
    }

    if let Some(&progress_bar) = matches.get_one::<bool>("progress-bar") {
        config.set_progress_bar(progress_bar);
    }

    if let Some(&padding) = matches.get_one::<u16>("padding") {
        config.set_padding(padding);
    }

    // Handle time parsing from command-line
    if let Some(args) = matches.get_many::<TimeValue>("time") {
        let mut total_secs: u32 = 0;

        for TimeValue { value, unit } in args {
            let secs = match unit {
                TimeUnit::Hours => value * 3600,
                TimeUnit::Minutes => value * 60,
                TimeUnit::Seconds => *value,
            };
            total_secs += secs as u32;
        }

        app.set_duration(Duration::new(total_secs as u64, 0));
    }

    // Set configuration for the app and run it
    app.set_config(config);
    run_app(app)
}

// Constructs the CLI arguments
fn command(config: &Config) -> Command {
    Command::new("brb")
        .version(env!("CARGO_PKG_VERSION"))
        .group(ArgGroup::new("customize").multiple(true))
        .next_help_heading("Customize")
        .args([
            // Set custom text
            Arg::new("text")
                .short('t')
                .long("text")
                .action(ArgAction::Set)
                .help("The text to display below the time")
                .group("customize"),
            // Enable/disable the chat
            Arg::new("chat")
                .long("chat")
                .value_parser(clap::value_parser!(bool))
                .default_value(if config.is_chat() { "true" } else { "false" })
                .help("Show the chat")
                .group("customize"),
            // Enable/disable current song display
            Arg::new("song-display")
                .long("song-display")
                .value_parser(clap::value_parser!(bool))
                .default_value(if config.is_song_display() {
                    "true"
                } else {
                    "false"
                })
                .help("Show the current song using spotic")
                .group("customize"),
            // Set the twitch channel
            Arg::new("twitch")
                .long("twitch")
                .action(ArgAction::Set)
                .help("The Twitch channel for chat integration")
                .group("customize"),
            // Color argument, either color name or RGB value
            Arg::new("color")
                .long("color")
                .action(ArgAction::Set)
                .value_parser(color_arg_parser)
                .help("The accent color, either NAME like 'red' or RGB like '255,0,0'")
                .value_name("NAME | RGB")
                .group("customize"),
            // Hide the timer after time is up
            Arg::new("hide-timer")
                .long("hide-timer")
                .value_parser(clap::value_parser!(bool))
                .default_value(if config.is_hide_timer() {
                    "true"
                } else {
                    "false"
                })
                .help("Hide the timer when it is finished")
                .group("customize"),
            // Enable/disable the progress bar
            Arg::new("progress-bar")
                .long("progress-bar")
                .value_parser(clap::value_parser!(bool))
                .default_value(if config.is_progress_bar() {
                    "true"
                } else {
                    "false"
                })
                .help("Display a progress bar of the timer's progress")
                .group("customize"),
            // Set outer padding
            Arg::new("padding")
                .long("padding")
                .value_parser(clap::value_parser!(u16))
                .help("Set the outer padding")
                .group("customize"),
            // The positional time arguments "1h 2m 3s"
            Arg::new("time")
                .help("Time arguments in the format [t]h, [t]m, or [t]s")
                .action(ArgAction::Append)
                .num_args(0..)
                .value_name("TIME")
                .value_parser(time_arg_parser), // We use a custom parser here
        ])
        .group(ArgGroup::new("info").multiple(true))
        .next_help_heading("Info")
        .args([Arg::new("dir")
            .long("dir")
            .action(ArgAction::SetTrue)
            .help("Display where the config file should be located")
            .group("info")])
}

// Custom parser for time arguments
fn time_arg_parser(arg: &str) -> Result<TimeValue, String> {
    /* Split the string at the last character in the string, the first part is the time value "13"
     * and the last part is the time unit character "h", "m" or "s"
     */
    let (value_str, unit_str) = arg.split_at(arg.len() - 1);

    // Match the time unit character to the corresponding TimeUnit enum variant
    let unit = match unit_str {
        "h" => TimeUnit::Hours,
        "m" => TimeUnit::Minutes,
        "s" => TimeUnit::Seconds,
        // Return a CLI parsing error if the character is not valid
        _ => return Err("Time arguments must end with 'h', 'm', or 's' suffix".to_string()),
    };

    // If no value is provided
    if value_str.is_empty() {
        return Err(format!("Missing time amount for '{}' time unit", unit_str));
    }

    // Try parsing the value to a u64
    let value = value_str.parse::<u64>().map_err(|_| {
        // Return a CLI parsing error if the value is not a valid number
        format!(
            "Invalid time amount '{}' for '{}' time unit",
            value_str, unit_str
        )
    })?;

    // If everything went well return a TimeValue instance
    Ok(TimeValue { value, unit })
}

// Custom parser for color arguments
fn color_arg_parser(arg: &str) -> Result<TomlColor, String> {
    // Try to map the arg to a color name
    if let Some(named_color) = TomlColor::from_name(arg) {
        return Ok(named_color);
    }

    // Check if the argument contains separators
    if !arg.contains(',') {
        return Err("Invalid color name".to_string());
    }

    // Check if argument starts or ends with comma
    if arg.starts_with(',') || arg.ends_with(',') {
        return Err("Invalid RGB color format, must be 'R,G,B'".to_string());
    }

    // Parse the RGB values
    let values: Vec<u8> = arg
        .split(',')
        .map(|v| {
            v.trim()
                .parse::<u8>()
                .map_err(|_| format!("Invalid value '{}', must be a number between 0 and 255", v))
        })
        .collect::<Result<Vec<u8>, String>>()?;

    let value_count = values.len();

    // Make sure there are exactly 3 values
    let [r, g, b]: [u8; 3] = values.try_into().map_err(|_| {
        format!(
            "Too many RGB values, must be 3, {} were provided",
            value_count
        )
    })?;

    // Return the parsed TomlColor
    Ok(TomlColor::Rgb { r, g, b })
}

// Output the config dir
fn output_dir() -> Result<()> {
    let config_dir = Config::get_config_dir()?
        .into_os_string()
        .into_string()
        .map_err(|_| {
            io::Error::new(
                io::ErrorKind::Other,
                "Failed converting config directory string",
            )
        })?;

    println!("{}", config_dir);

    Ok(())
}

// Run the App
fn run_app(mut app: App) -> Result<()> {
    let mut terminal = ratatui::init();
    app.run(&mut terminal).context("Failed initializing UI")?;
    ratatui::restore();
    Ok(())
}
