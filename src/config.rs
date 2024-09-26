/*
*  TODO:
*  Make the config default value process a little cleaner
*/

use ratatui::style::Color;
use serde::Deserialize;
use serde_with::{serde_as, DefaultOnError};
use std::{
    fs,
    io::{self, ErrorKind},
};

// A color which is deserialized from the toml config file
#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum TomlColor {
    Rgb { r: u8, g: u8, b: u8 }, // An RGB color value
    Name(String), // The name of a color preset, such as "red", "yellow", "white", ...
}

/*
* These are the configuration values for the program.
* We use serde_with to insert default values in case they were not provided found during the
* deserialization process
*/
#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_color")] // Default is "default_color"
    #[serde_as(deserialize_as = "DefaultOnError")] // Use default if deserialization fails
    color: Option<TomlColor>, // The UI accent color
    #[serde(default = "default_text")] // Default is "default_text"
    text: String, // The text which is displayed in the middle
    #[serde(default)] // Default is None
    twitch_channel: Option<String>,
    #[serde(default)] // Default is false
    chat: bool, // Whether to display the chat
    #[serde(default)] // Default is false
    hide_timer: bool, // Whether to hide the countdown when it's done
    #[serde(default)] // Default is false
    progress_bar: bool, // Whether to display the progress bar
}

// This function will return the default color, which is white
fn default_color() -> Option<TomlColor> {
    Some(TomlColor::Name("White".to_string())) // Default to White color
}

// This function will return the default text, which is "Be right back"
fn default_text() -> String {
    "Be right back".to_string()
}

// Get default config
impl Default for Config {
    fn default() -> Self {
        Self {
            color: default_color(),
            text: default_text(),
            twitch_channel: None,
            chat: false,
            hide_timer: false,
            progress_bar: true,
        }
    }
}

impl Config {
    // Load the configuration file
    pub fn load() -> io::Result<Self> {
        // Get the OS specific configuration directory
        let mut config_dir = dirs::config_dir()
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Failed to get config directory"))?;

        // Append "/brb/brb.toml"
        config_dir.push("brb");
        config_dir.push("brb.toml");

        // If the config file exists
        if config_dir.is_file() {
            // Read the file
            let config_str = fs::read_to_string(config_dir)?;

            // Deserialize it
            match toml::from_str::<Config>(&config_str) {
                Ok(mut config) => {
                    config.merge_with_defaults();
                    Ok(config)
                }
                Err(err) => Err(io::Error::new(ErrorKind::InvalidData, err.to_string())),
            }
        } else {
            // Otherwise return the default config
            Ok(Self::default())
        }
    }

    // Here we merge the config file values with default values if they are missing
    fn merge_with_defaults(&mut self) {
        let default = Self::default();

        if self.color.is_none() {
            self.color = default.color;
        }
        if self.text.is_empty() {
            self.text = default.text;
        }
        if self.twitch_channel.is_none() {
            self.twitch_channel = default.twitch_channel;
        }
    }

    // Get the color from the config
    pub fn get_color(&self) -> Color {
        match &self.color {
            // If the color is deserializeable as a RGB color
            Some(TomlColor::Rgb { r, g, b }) => Color::Rgb(*r, *g, *b),
            // If the color is a color preset name
            Some(TomlColor::Name(name)) => match name.to_lowercase().as_str() {
                "black" => Color::Black,
                "red" => Color::Red,
                "green" => Color::Green,
                "yellow" => Color::Yellow,
                "blue" => Color::Blue,
                "magenta" => Color::Magenta,
                "cyan" => Color::Cyan,
                "white" => Color::White,
                _ => Color::White,
            },
            None => Color::White,
        }
    }

    /*
     * Remaining functions are simple setters and getters
     */

    pub fn set_text(&mut self, text: String) {
        self.text = text
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

    pub fn set_hide_timer(&mut self, hide_timer: bool) {
        self.hide_timer = hide_timer
    }

    pub fn is_hide_timer(&self) -> bool {
        self.hide_timer
    }

    pub fn set_progress_bar(&mut self, progress_bar: bool) {
        self.progress_bar = progress_bar
    }

    pub fn is_progress_bar(&self) -> bool {
        self.progress_bar
    }
}
