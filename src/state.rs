use std::io;
use std::time::{Duration, Instant};

use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Style, Stylize},
    text::Line,
    widgets::{block::Title, Block, Borders, Widget},
    DefaultTerminal, Frame,
};
use tui_big_text::{BigText, PixelSize};

#[derive(Debug)]
pub struct App {
    color: Color,
    start_time: Option<Instant>,
    original_duration: Option<Duration>,
    remaining_time: Option<Duration>,
    text: String,
    chat: bool,
    exit: bool,
}

impl Default for App {
    fn default() -> Self {
        Self {
            color: Color::Red,
            start_time: None,
            original_duration: None,
            remaining_time: None,
            text: "Be right back".to_string(),
            chat: false,
            exit: false,
        }
    }
}

impl App {
    pub fn new(duration: Duration, text: String) -> Self {
        Self {
            color: Color::White,
            start_time: Some(Instant::now()),
            original_duration: Some(duration),
            remaining_time: Some(duration),
            text,
            chat: false,
            exit: false,
        }
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.original_duration = Some(duration);
        self.remaining_time = Some(duration);
        self.start_time = Some(Instant::now());
    }

    pub fn set_text(&mut self, text: String) {
        self.text = text;
    }

    pub fn set_chat(&mut self, chat: bool) {
        self.chat = chat;
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let redraw_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        while !self.exit {
            self.handle_events()?;

            let now = Instant::now();
            if now.duration_since(last_tick) >= redraw_rate {
                self.update_time();
                last_tick = now; // Update the last tick time to the current time
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn update_time(&mut self) {
        if let (Some(duration), Some(start_time)) = (self.original_duration, self.start_time) {
            let elapsed = start_time.elapsed();
            let remaining_time = if elapsed >= duration {
                Duration::ZERO
            } else {
                duration - elapsed
            };
            self.remaining_time = Some(remaining_time);
        }
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    fn handle_events(&mut self) -> io::Result<()> {
        while event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                self.handle_key_event(key_event);
            }
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let KeyCode::Char('q') = key_event.code {
            self.exit();
        }
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let text_lines: Vec<Line> = self.text.split('\n').map(Line::from).collect();

        let horizontal_constraints = if self.chat {
            vec![
                Constraint::Fill(1),
                Constraint::Ratio(2, 3),
                Constraint::Ratio(1, 3),
                Constraint::Fill(1),
            ]
        } else {
            vec![
                Constraint::Fill(1),
                Constraint::Ratio(3, 3),
                Constraint::Fill(1),
            ]
        };

        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraints)
            .split(area);

        let vertical_constraints = if self.remaining_time.is_some() {
            vec![
                Constraint::Fill(1),
                Constraint::Max(8),
                Constraint::Max(4 * text_lines.len() as u16),
                Constraint::Fill(1),
            ]
        } else {
            vec![
                Constraint::Fill(1),
                Constraint::Max(4 * text_lines.len() as u16),
                Constraint::Fill(1),
            ]
        };

        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints)
            .split(horizontal_layout[1]);

        if let Some(duration) = &self.remaining_time {
            let time_str = format_duration(*duration);
            let time_display = BigText::builder()
                .pixel_size(PixelSize::Full)
                .style(Style::new().fg(self.color))
                .lines(vec![time_str.into()])
                .centered()
                .build();

            let time_area = vertical_layout[1];
            time_display.render(time_area, buf);
        }

        let text_display = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new().white())
            .lines(text_lines)
            .centered()
            .build();

        let text_area = if self.remaining_time.is_some() {
            vertical_layout[2]
        } else {
            vertical_layout[1]
        };

        text_display.render(text_area, buf);

        let chat_title = Title::from(" Chat ".fg(self.color)).alignment(Alignment::Center);
        let chat_display = Block::default().title(chat_title).borders(Borders::ALL);
        let chat_area = horizontal_layout[2];

        chat_display.render(chat_area, buf);
    }
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}", mins, secs)
}
