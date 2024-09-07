use std::io;
use std::time::Duration;

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    widgets::Widget,
    DefaultTerminal, Frame,
};
use tui_big_text::{BigText, PixelSize};

#[derive(Debug)]
pub struct App {
    time: Option<Duration>,
    text: String,
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            time: None,
            text: "Be right back".to_string(),
            exit: false,
        }
    }
}

impl App {
    pub fn new(time: Duration, text: String) -> Self {
        Self {
            time: Some(time),
            text,
            exit: false,
        }
    }

    pub fn set_time(&mut self, time: Duration) {
        self.time = Some(time);
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            self.update_time();
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }

        Ok(())
    }

    fn update_time(&mut self) {
        // Time update functionality removed as per your requirement
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if let Event::Key(key_event) = event::read()? {
            self.handle_key_event(key_event);
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let constraints = if self.time.is_some() {
            vec![
                Constraint::Fill(1), // Empty space at the top
                Constraint::Max(8),  // Space for time if it exists
                Constraint::Max(4),  // Space for text
                Constraint::Fill(1), // Empty space at the bottom
            ]
        } else {
            vec![
                Constraint::Fill(1), // Empty space at the top
                Constraint::Max(4),  // Space for text only
                Constraint::Fill(1), // Empty space at the bottom
            ]
        };

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);

        // Render time display only if it's Some
        if let Some(duration) = &self.time {
            let time_str = format_duration(*duration);
            let time_display = BigText::builder()
                .pixel_size(PixelSize::Full)
                .style(Style::new().red())
                .lines(vec![time_str.into()])
                .centered()
                .build();

            let time_area = layout[1]; // The area for the time display
            time_display.render(time_area, buf);
        }

        // Render the text display
        let text_display = BigText::builder()
            .pixel_size(PixelSize::HalfHeight)
            .style(Style::new().white())
            .lines(vec![self.text.clone().into()])
            .centered()
            .build();

        let text_area = if self.time.is_some() {
            layout[2]
        } else {
            layout[1]
        };

        text_display.render(text_area, buf);
    }
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}", mins, secs)
}
