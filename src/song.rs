use std::process::Command;

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget, Wrap},
};

pub struct SongDisplay {
    current_song: String,
}

impl SongDisplay {
    pub fn new() -> Self {
        Self {
            current_song: "Getting current song ...".to_string(),
        }
    }

    pub fn poll_song(&mut self) {
        let mut command = Command::new("sc");
        command.arg("current");

        let text = match command.output() {
            Ok(o) => String::from_utf8(o.stdout).map_err(|_| "Failed reading output"),
            Err(_) => Err("Failed running spotic"),
        };

        match text {
            Ok(s) => self.current_song = s,
            Err(e) => self.current_song = format!("Error: {}", e),
        }
    }
}

impl Widget for &SongDisplay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let song = Paragraph::new(self.current_song.clone()).wrap(Wrap { trim: true });

        song.render(area, buf);
    }
}
