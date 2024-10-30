use std::{
    io::Read,
    process::{Child, Command, Stdio},
};

use ratatui::{
    buffer::Buffer,
    layout::Rect,
    widgets::{Paragraph, Widget, Wrap},
};

pub struct SongDisplay {
    current_song: String,
    child: Option<Child>,
}

impl SongDisplay {
    pub fn new() -> Self {
        Self {
            current_song: "Getting current song ...".to_string(),
            child: None,
        }
    }

    pub fn poll_song(&mut self) {
        if let Some(mut child) = self.child.take() {
            if let Some(mut stdout) = child.stdout.take() {
                let mut buf = String::new();

                self.current_song = match stdout.read_to_string(&mut buf) {
                    Ok(_) => buf,
                    Err(_) => "Failed reading output".to_string(),
                };

                self.child = Some(child);
                return;
            }
        }

        match Command::new("sc")
            .arg("current")
            .stdout(Stdio::piped())
            .spawn()
        {
            Ok(child) => self.child = Some(child),
            Err(_) => self.current_song = "Failed running spotic".to_string(),
        }
    }

    pub fn stop(&mut self) {
        if let Some(mut c) = self.child.take() {
            let _ = c.kill();
        }
    }
}

impl Widget for &SongDisplay {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let song = Paragraph::new(self.current_song.clone()).wrap(Wrap { trim: true });

        song.render(area, buf);
    }
}
