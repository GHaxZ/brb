use anyhow::{Context, Result};
use ratatui::style::Color;
use serde::Deserialize;
use serde_with::{serde_as, DefaultOnError};
use std::{fs, path::PathBuf};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
// A color which is deserialized from the toml config file
pub enum TomlColor {
    Rgb { r: u8, g: u8, b: u8 }, // An RGB color value
    Name(String), // The name of a color preset, such as "red", "yellow", "white", ...
}

impl TomlColor {
    // Get a TomlColor from a color name, or None if this name was not found
    pub fn from_name(name: &str) -> Option<Self> {
        let name = name.to_lowercase();

        match name.as_str() {
            "black" => Some(Self::Name(name)),
            "red" => Some(Self::Name(name)),
            "green" => Some(Self::Name(name)),
            "yellow" => Some(Self::Name(name)),
            "blue" => Some(Self::Name(name)),
            "magenta" => Some(Self::Name(name)),
            "cyan" => Some(Self::Name(name)),
            "white" => Some(Self::Name(name)),
            _ => None,
        }
    }
}

/*
* The default values are set here
*/
const DEFAULT_COLOR: &str = "white";
const DEFAULT_TEXT: &str = "Be right back";
const DEFAULT_TWITCH_CHANNEL: Option<String> = None;
const DEFAULT_CHAT: bool = false;
const DEFAULT_SONG_DISPLAY: bool = false;
const DEFAULT_HIDE_TIMER: bool = true;
const DEFAULT_PROGRESS_BAR: bool = true;
const DEFAULT_PADDING: u16 = 1;
const DEFAULT_START_COMMANDS: Vec<String> = vec![];
const DEFAULT_EXIT_COMMANDS: Vec<String> = vec![];

// Implement Default by calling the default_color() function. We have to do this, because this will
// be used in case the deserialization fails.
impl Default for TomlColor {
    fn default() -> Self {
        default_color()
    }
}

/*
* These are the configuration values for the program.
*
* We use serde_with to insert default values in case they were not provided during the
* deserialization process
*
* We do this by calling a "default_*()" function for each of these values
*/
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_color")]
    #[serde_as(deserialize_as = "DefaultOnError")] // Use default if deserialization fails
    color: TomlColor, // The UI accent color
    #[serde(default = "default_text")]
    text: String, // The text which is displayed in the middle
    #[serde(default = "default_twitch_channel")]
    twitch_channel: Option<String>,
    #[serde(default = "default_chat")]
    chat: bool, // Whether to display the chat
    #[serde(default = "default_song_display")]
    song_display: bool, // Whether to display the current song
    #[serde(default = "default_hide_timer")]
    hide_timer: bool, // Whether to hide the countdown when it's done
    #[serde(default = "default_progress_bar")]
    progress_bar: bool, // Whether to display the progress bar
    #[serde(default = "default_padding")]
    padding: u16, // The amount of outer padding
    #[serde(default = "default_start_commands")]
    start_commands: Vec<String>, // Commands run when starting brb
    #[serde(default = "default_exit_commands")]
    exit_commands: Vec<String>, // Commands run when exiting brb
}

// This function will return the default color
fn default_color() -> TomlColor {
    TomlColor::Name(DEFAULT_COLOR.to_string())
}

// This function will return the default text
fn default_text() -> String {
    DEFAULT_TEXT.to_string()
}

// This function will return the default twitch channel
fn default_twitch_channel() -> Option<String> {
    DEFAULT_TWITCH_CHANNEL
}

// This function will return the default chat
fn default_chat() -> bool {
    DEFAULT_CHAT
}

// This function will return the default song display
fn default_song_display() -> bool {
    DEFAULT_SONG_DISPLAY
}

// This function will return the default hide timer
fn default_hide_timer() -> bool {
    DEFAULT_HIDE_TIMER
}

// This function will return the default progress bar
fn default_progress_bar() -> bool {
    DEFAULT_PROGRESS_BAR
}

// This function will return the default padding
fn default_padding() -> u16 {
    DEFAULT_PADDING
}

// This function will return the default start commands
fn default_start_commands() -> Vec<String> {
    DEFAULT_START_COMMANDS
}

// This function will return the default start commands
fn default_exit_commands() -> Vec<String> {
    DEFAULT_EXIT_COMMANDS
}

// Get default config
impl Default for Config {
    fn default() -> Self {
        Self {
            color: default_color(),
            text: default_text(),
            twitch_channel: default_twitch_channel(),
            chat: default_chat(),
            song_display: default_song_display(),
            hide_timer: default_hide_timer(),
            progress_bar: default_progress_bar(),
            padding: default_padding(),
            start_commands: default_start_commands(),
            exit_commands: default_exit_commands(),
        }
    }
}

impl Config {
    // Load the configuration file
    pub fn load() -> Result<Self> {
        let config_dir = Self::get_config_dir().context("Failed getting config directory")?;

        // If the config file exists
        if config_dir.is_file() {
            // Read the file
            let config_str =
                fs::read_to_string(config_dir).context("Failed reading config file")?;

            // Deserialize it
            toml::from_str::<Config>(&config_str).context("Failed deserializing configuration file")
        } else {
            // Otherwise return the default config
            Ok(Self::default())
        }
    }

    pub fn get_config_dir() -> Result<PathBuf> {
        // Get the OS specific configuration directory
        let mut config_dir =
            dirs::config_dir().context("Failed getting OS conventional config directory")?;

        // Append "/brb/brb.toml"
        config_dir.push("brb");
        config_dir.push("brb.toml");

        Ok(config_dir)
    }

    // Map a color name to an actual Color variant
    fn map_color_name(name: &str) -> Color {
        match name {
            "black" => Color::Black,
            "red" => Color::Red,
            "green" => Color::Green,
            "yellow" => Color::Yellow,
            "blue" => Color::Blue,
            "magenta" => Color::Magenta,
            "cyan" => Color::Cyan,
            "white" => Color::White,
            _ => Self::map_color_name(DEFAULT_COLOR),
        }
    }

    // Get the color from the config
    pub fn get_color(&self) -> Color {
        match &self.color {
            // If the color is deserializeable as a RGB color
            TomlColor::Rgb { r, g, b } => Color::Rgb(*r, *g, *b),
            // If the color is a color preset name
            TomlColor::Name(name) => Self::map_color_name(name.to_lowercase().as_str()),
        }
    }

    /*
     * Remaining functions are simple setters and getters
     */

    pub fn set_color(&mut self, color: TomlColor) {
        self.color = color;
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn get_text(&self) -> String {
        self.text.clone()
    }

    pub fn set_twitch_channel(&mut self, twitch_channel: String) {
        self.twitch_channel = Some(twitch_channel);
    }

    pub fn get_twitch_channel(&self) -> Option<String> {
        self.twitch_channel.clone()
    }

    pub fn set_chat(&mut self, chat: bool) {
        self.chat = chat;
    }

    pub fn is_chat(&self) -> bool {
        self.chat
    }

    pub fn set_song_display(&mut self, song_display: bool) {
        self.song_display = song_display;
    }

    pub fn is_song_display(&self) -> bool {
        self.song_display
    }

    pub fn set_hide_timer(&mut self, hide_timer: bool) {
        self.hide_timer = hide_timer;
    }

    pub fn is_hide_timer(&self) -> bool {
        self.hide_timer
    }

    pub fn set_progress_bar(&mut self, progress_bar: bool) {
        self.progress_bar = progress_bar;
    }

    pub fn is_progress_bar(&self) -> bool {
        self.progress_bar
    }

    pub fn set_padding(&mut self, padding: u16) {
        self.padding = padding
    }

    pub fn get_padding(&self) -> u16 {
        self.padding
    }

    pub fn get_start_commands(&self) -> Vec<String> {
        self.start_commands.clone()
    }

    pub fn get_exit_commands(&self) -> Vec<String> {
        self.exit_commands.clone()
    }
}
