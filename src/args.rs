use std::{io, time::Duration};

use crate::state::App;
use clap::{Arg, ArgAction, ArgGroup, Command};

#[derive(Clone)]
enum TimeUnit {
    Seconds,
    Minutes,
    Hours,
}

#[derive(Clone)]
struct TimeValue {
    value: u64,
    unit: TimeUnit,
}

pub fn parse() -> io::Result<()> {
    let mut app = App::default();

    let matches = command().get_matches();

    if let Some(text) = matches.get_one::<String>("text") {
        app.set_text(text.clone());
    }

    match matches.get_flag("chat") {
        chat => {
            app.set_chat(chat);
        }
    }

    match matches.get_flag("hide-timer") {
        hide_timer => {
            app.set_hide_timer(hide_timer);
        }
    }

    match matches.get_flag("progress-bar") {
        progress_bar => {
            app.set_progress_bar(progress_bar);
        }
    }

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

    run_app(app)
}

fn command() -> Command {
    Command::new("brb")
        .group(ArgGroup::new("customize").multiple(true))
        .next_help_heading("Customize")
        .args([
            Arg::new("text")
                .short('t')
                .long("text")
                .action(ArgAction::Set)
                .help("The text to display below the time")
                .group("customize"),
            Arg::new("chat")
                .long("chat")
                .action(ArgAction::SetTrue)
                .help("Show the chat")
                .group("customize"),
            Arg::new("hide-timer")
                .long("hide-timer")
                .action(ArgAction::SetTrue)
                .help("Hide the timer when it is finished")
                .group("customize"),
            Arg::new("progress-bar")
                .long("progress-bar")
                .action(ArgAction::SetTrue)
                .help("Display a progress bar of the timers progress")
                .group("customize"),
            Arg::new("time")
                .help("Time arguments in the format [t]h, [t]m, or [t]s")
                .action(ArgAction::Append)
                .num_args(0..)
                .value_name("TIME")
                .value_parser(time_arg_parser),
        ])
}

fn time_arg_parser(arg: &str) -> Result<TimeValue, String> {
    let (value_str, unit_str) = arg.split_at(arg.len() - 1);

    let unit = match unit_str {
        "h" => TimeUnit::Hours,
        "m" => TimeUnit::Minutes,
        "s" => TimeUnit::Seconds,
        _ => {
            return Err(format!(
                "Time arguments must end with 'h', 'm', or 's' suffix."
            ))
        }
    };

    if value_str.is_empty() {
        return Err(format!("Missing time amount for '{}' time unit.", unit_str));
    }

    let value = value_str.parse::<u64>().map_err(|_| {
        format!(
            "Invalid time amount '{}' for '{}' time unit.",
            value_str, unit_str
        )
    })?;

    Ok(TimeValue { value, unit })
}

pub fn run_app(mut app: App) -> io::Result<()> {
    let mut terminal = ratatui::init();
    app.run(&mut terminal)?;
    ratatui::restore();
    Ok(())
}
