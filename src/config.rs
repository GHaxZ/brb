use ratatui::style::Color;
use serde::Deserialize;
use serde_with::{serde_as, DefaultOnError};
use std::{
    fs,
    io::{self, ErrorKind},
};

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum TomlColor {
    Rgb { r: u8, g: u8, b: u8 },
    Name(String),
}

#[serde_as]
#[derive(Debug, Deserialize)]
pub struct Config {
    #[serde(default = "default_color")]
    #[serde_as(deserialize_as = "DefaultOnError")]
    color: Option<TomlColor>,
    #[serde(default = "default_text")]
    text: String,
    #[serde(default)]
    twitch_channel: Option<String>,
    #[serde(default)]
    chat: bool,
    #[serde(default)]
    hide_timer: bool,
    #[serde(default)]
    progress_bar: bool,
}

fn default_color() -> Option<TomlColor> {
    Some(TomlColor::Name("White".to_string())) // Default to White color
}

fn default_text() -> String {
    "Be right back".to_string()
}

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
    pub fn load() -> io::Result<Self> {
        let mut config_dir = dirs::config_dir()
            .ok_or_else(|| io::Error::new(ErrorKind::NotFound, "Failed to get config directory"))?;

        config_dir.push("brb");
        config_dir.push("brb.toml");

        if config_dir.is_file() {
            let config_str = fs::read_to_string(config_dir)?;

            match toml::from_str::<Config>(&config_str) {
                Ok(mut config) => {
                    config.merge_with_defaults();
                    Ok(config)
                }
                Err(err) => Err(io::Error::new(ErrorKind::InvalidData, err.to_string())),
            }
        } else {
            Ok(Self::default())
        }
    }

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

    pub fn get_color(&self) -> Color {
        match &self.color {
            Some(TomlColor::Rgb { r, g, b }) => Color::Rgb(*r, *g, *b),
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
