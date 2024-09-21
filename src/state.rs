use ratatui::widgets::{Gauge, Padding};
use ratatui::{
    buffer::Buffer,
    crossterm::event::{self, Event, KeyCode, KeyEvent},
    layout::{Constraint, Direction, Layout, Rect},
    style::{Style, Stylize},
    text::Line,
    widgets::{Block, Borders, Widget},
    DefaultTerminal, Frame,
};
use std::io;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tui_big_text::{BigText, PixelSize};

use crate::chat::TwitchChat;
use crate::config::Config;

pub struct App {
    config: Config,
    chat: Option<TwitchChat>,
    start_time: Option<Instant>,
    original_duration: Option<Duration>,
    remaining_time: Option<Duration>,
    exit: bool,
}

#[allow(clippy::derivable_impls)]
impl Default for App {
    fn default() -> Self {
        Self {
            config: Config::default(),
            chat: None,
            start_time: None,
            original_duration: None,
            remaining_time: None,
            exit: false,
        }
    }
}

impl App {
    pub fn set_config(&mut self, config: Config) {
        self.config = config
    }

    pub fn set_duration(&mut self, duration: Duration) {
        self.original_duration = Some(duration);
        self.remaining_time = Some(duration);
        self.start_time = Some(Instant::now());
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let rt = Runtime::new().unwrap();

        if self.config.is_chat() {
            if let Some(channel) = self.config.get_twitch_channel() {
                self.chat = Some(TwitchChat::new(self.config.get_color(), channel));

                rt.block_on(async {
                    if let Some(chat) = self.chat.as_mut() {
                        chat.start().unwrap();
                    }
                });
            }
        }

        let redraw_rate = Duration::from_millis(100);
        let mut last_tick = Instant::now();

        while !self.exit {
            self.handle_events()?;

            let now = Instant::now();
            if now.duration_since(last_tick) >= redraw_rate {
                self.update_time();
                last_tick = now;

                if let Some(chat) = self.chat.as_mut() {
                    chat.poll_messages();
                }
            }

            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    fn update_time(&mut self) {
        if let (Some(duration), Some(start_time)) = (self.original_duration, self.start_time) {
            let elapsed = start_time.elapsed();
            if elapsed >= duration {
                if self.config.is_hide_timer() {
                    self.remaining_time = None;
                } else {
                    self.remaining_time = Some(Duration::ZERO);
                }
            } else {
                self.remaining_time = Some(duration - elapsed);
            };
        }
    }

    fn time_percentage(&self) -> Option<u16> {
        if let (Some(start_time), Some(original_duration)) =
            (&self.start_time, &self.original_duration)
        {
            let elapsed = start_time.elapsed().as_secs_f64();
            let total = original_duration.as_secs_f64();
            Some(((elapsed / total) * 100.0).min(100.0) as u16)
        } else {
            None
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
        let text = self.config.get_text();
        let text_lines: Vec<Line> = text.split('\n').map(Line::from).collect();

        let horizontal_constraints =
            if self.config.is_chat() && self.config.get_twitch_channel().is_some() {
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
                Constraint::Max(3),
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
                .style(Style::new().fg(self.config.get_color()))
                .lines(vec![time_str.into()])
                .centered()
                .build();

            let time_area = vertical_layout[1];
            time_display.render(time_area, buf);

            if let Some(percentage) = &self.time_percentage() {
                if self.config.is_progress_bar() {
                    let progress_display = Gauge::default()
                        .block(
                            Block::default()
                                .borders(Borders::NONE)
                                .padding(Padding::uniform(1)),
                        )
                        .gauge_style(Style::new().fg(self.config.get_color()))
                        .use_unicode(true)
                        .percent(*percentage);

                    let progress_area = vertical_layout[4];

                    progress_display.render(progress_area, buf);
                }
            }
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

        if let Some(chat) = &self.chat {
            chat.render(horizontal_layout[2], buf);
        }
    }
}

fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}", mins, secs)
}
