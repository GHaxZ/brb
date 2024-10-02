use std::{io, time::Duration};

use crate::{config::Config, state::App};
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
pub fn parse() -> io::Result<()> {
    // Create the app and load the config file
    let mut app = App::default();
    let mut config = Config::load()?;

    let matches = command(&config).get_matches();

    // Check CLI arguments and update the config if necessary

    if let Some(text) = matches.get_one::<String>("text") {
        config.set_text(text.clone());
    }

    if let Some(&chat) = matches.get_one::<bool>("chat") {
        config.set_chat(chat);
    }

    if let Some(&hide_timer) = matches.get_one::<bool>("hide-timer") {
        config.set_hide_timer(hide_timer);
    }

    if let Some(&progress_bar) = matches.get_one::<bool>("progress-bar") {
        config.set_progress_bar(progress_bar);
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
        .group(ArgGroup::new("info").multiple(true))
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
            // The positional time arguments "1h 2m 3s"
            Arg::new("time")
                .help("Time arguments in the format [t]h, [t]m, or [t]s")
                .action(ArgAction::Append)
                .num_args(0..)
                .value_name("TIME")
                .value_parser(time_arg_parser), // We use a custom parser here
        ])
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
        _ => return Err("Time arguments must end with 'h', 'm', or 's' suffix.".to_string()),
    };

    // If no value is provided
    if value_str.is_empty() {
        return Err(format!("Missing time amount for '{}' time unit.", unit_str));
    }

    // Try parsing the value to a u64
    let value = value_str.parse::<u64>().map_err(|_| {
        // Return a CLI parsing error if the value is not a valid number
        format!(
            "Invalid time amount '{}' for '{}' time unit.",
            value_str, unit_str
        )
    })?;

    // If everything went well return a TimeValue instance
    Ok(TimeValue { value, unit })
}

// Run the App
fn run_app(mut app: App) -> io::Result<()> {
    let mut terminal = ratatui::init();
    app.run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}
