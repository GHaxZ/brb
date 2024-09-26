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
    config: Config,                      // The config used for this App
    chat: Option<TwitchChat>,            // The TwitchChat widget if enabled
    start_time: Option<Instant>,         // The start time of the countdown
    original_duration: Option<Duration>, // The original duration of the countdown
    remaining_time: Option<Duration>,    // The remaining time of the countdown
    exit: bool,                          // Exit if this is true
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

    // Run the app
    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        // Create a new tokio runtime which is used for twitch chat
        let rt = Runtime::new().unwrap();

        // If the chat is enabled
        if self.config.is_chat() {
            // If a twitch channel was configured
            if let Some(channel) = self.config.get_twitch_channel() {
                // Create a new twitch chat widget
                self.chat = Some(TwitchChat::new(self.config.get_color(), channel));

                // Run the chat on a blocking tokio task
                rt.block_on(async {
                    if let Some(chat) = self.chat.as_mut() {
                        chat.start().unwrap();
                    }
                });
            }
        }

        // How often the UI should be forcefully redrawn
        let redraw_rate = Duration::from_millis(100);
        // Last redraw time
        let mut last_tick = Instant::now();

        // While we don't want to exit
        while !self.exit {
            // Handle events such as key events
            self.handle_events()?;

            // Current time
            let now = Instant::now();

            // If a redraw should happen
            if now.duration_since(last_tick) >= redraw_rate {
                // Update the time
                self.update_time();
                last_tick = now;

                // Poll chat messages
                if let Some(chat) = self.chat.as_mut() {
                    chat.poll_messages();
                }
            }

            // Draw the UI
            terminal.draw(|frame| self.draw(frame))?;
        }

        Ok(())
    }

    // Update the time values
    fn update_time(&mut self) {
        // If a countdown is set
        if let (Some(duration), Some(start_time)) = (self.original_duration, self.start_time) {
            // How much time has elapsed since the countdown start
            let elapsed = start_time.elapsed();

            // If the countdown has finished
            if elapsed >= duration {
                // If the the timer is configured to be hidden
                if self.config.is_hide_timer() {
                    self.remaining_time = None;
                } else {
                    self.remaining_time = Some(Duration::ZERO);
                }
            } else { // Otherwise update the remaining time value
                self.remaining_time = Some(duration - elapsed);
            };
        }
    }

    // Caluclate how much of the time has elapsed in percent
    fn time_percentage(&self) -> Option<u16> {
        // If a countdown is set
        if let (Some(start_time), Some(original_duration)) =
            (&self.start_time, &self.original_duration)
        {
            let elapsed = start_time.elapsed().as_secs_f64();
            let total = original_duration.as_secs_f64();
            Some(((elapsed / total) * 100.0).min(100.0) as u16)
        } else { // Otherwise return None
            None
        }
    }

    // Draw the App
    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    // Handle events
    fn handle_events(&mut self) -> io::Result<()> {
        while event::poll(Duration::from_millis(50))? {
            if let Event::Key(key_event) = event::read()? {
                self.handle_key_event(key_event);
            }
        }
        Ok(())
    }

    // Specifically handle key input events
    fn handle_key_event(&mut self, key_event: KeyEvent) {
        if let KeyCode::Char('q') = key_event.code {
            self.exit();
        }
    }

    // Exit the App
    fn exit(&mut self) {
        self.exit = true;
    }
}

// Implement Widget for the App so it can be rendered
impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Split the text which should be displayed into multiple lines at newline characters
        let text = self.config.get_text();
        let text_lines: Vec<Line> = text.split('\n').map(Line::from).collect();

        // Layout constraints for horizontally aligned widgets
        let horizontal_constraints =
            // If the chat is enabled split the layout in a 2 to 1 ratio
            if self.chat.is_some() {
                vec![
                    Constraint::Fill(1),
                    Constraint::Ratio(2, 3),
                    Constraint::Ratio(1, 3),
                    Constraint::Fill(1),
                ]
            } else { // Otherwise give the other elements the entire width
                vec![
                    Constraint::Fill(1),
                    Constraint::Ratio(3, 3),
                    Constraint::Fill(1),
                ]
            };

        // Split the entire terminal area into a layout based on the constraints
        let horizontal_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(horizontal_constraints)
            .split(area);

        // Layout constraints for horizontally aligned widgets
        let vertical_constraints = 
        // If there is a remaining time we reserve some space for the countdown
        if self.remaining_time.is_some() {
            vec![
                Constraint::Fill(1),
                Constraint::Max(8),
                Constraint::Max(4 * text_lines.len() as u16), // Enough space for all text lines
                Constraint::Fill(1),
                Constraint::Max(3),
            ]
        } else { // Otherwise we allow the other elements to use this space
            vec![
                Constraint::Fill(1), 
                Constraint::Max(4 * text_lines.len() as u16), // Enough space for all text lines
                Constraint::Fill(1),
            ]
        };

        // Split a part of the horizontal layout based on the constraints
        let vertical_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(vertical_constraints)
            .split(horizontal_layout[1]);

        // If some time is remaining on the countdown
        if let Some(duration) = &self.remaining_time {
            // Format the time nicely
            let time_str = format_duration(*duration);

            // Display it using the BigText widget
            let time_display = BigText::builder()
                .pixel_size(PixelSize::Full)
                .style(Style::new().fg(self.config.get_color()))
                .lines(vec![time_str.into()])
                .centered()
                .build();

            // And finally render it at the correct position inside the vertical layout
            let time_area = vertical_layout[1];
            time_display.render(time_area, buf);

            // If we have a completion percentage
            if let Some(percentage) = &self.time_percentage() {
                // And if the progress bar is enabled
                if self.config.is_progress_bar() {
                    // Create a new "Gauge" widget
                    let progress_display = Gauge::default()
                        .block(
                            Block::default()
                                .borders(Borders::NONE)
                                .padding(Padding::uniform(1)),
                        )
                        .gauge_style(Style::new().fg(self.config.get_color()))
                        .use_unicode(true)
                        .percent(*percentage);

                    // And render it
                    let progress_area = vertical_layout[4];
                    progress_display.render(progress_area, buf);
                }
            }
        }

        // Create a BigText widget for the text
        let text_display = BigText::builder()
            .pixel_size(PixelSize::Quadrant)
            .style(Style::new().white())
            .lines(text_lines)
            .centered()
            .build();

        // And render it in the correct position depending on if the time is displayed
        let text_area = if self.remaining_time.is_some() {
            vertical_layout[2]
        } else {
            vertical_layout[1]
        };

        text_display.render(text_area, buf);

        // If we have a chat, render it
        if let Some(chat) = &self.chat {
            chat.render(horizontal_layout[2], buf);
        }
    }
}

// Helper function for formatting the time
fn format_duration(duration: Duration) -> String {
    let secs = duration.as_secs();
    let mins = secs / 60;
    let secs = secs % 60;
    format!("{:02}:{:02}", mins, secs)
}
