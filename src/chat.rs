/*
*   TODO:
*   Clean up the abomination that is line wrapping (please lord forgive me)
*/

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType, Borders, List, ListDirection, Padding, Widget},
};
use std::io;
use std::sync::{Arc, Mutex};
use tokio::runtime::{Builder, Runtime};
use tokio::sync::mpsc;
use twitch_irc::{
    login::StaticLoginCredentials,
    message::{RGBColor, ServerMessage},
    ClientConfig, SecureTCPTransport, TwitchIRCClient,
};

pub struct TwitchClient {
    runtime: Runtime,
    tx: mpsc::Sender<TwitchMessage>,
}

impl TwitchClient {
    pub fn new(tx: mpsc::Sender<TwitchMessage>) -> Self {
        Self {
            runtime: Builder::new_multi_thread()
                .worker_threads(1)
                .enable_all()
                .build()
                .unwrap(),
            tx,
        }
    }

    pub fn start(&mut self, channel: String) -> io::Result<()> {
        let tx = self.tx.clone();
        let config = ClientConfig::default();
        let (mut incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        self.runtime.spawn(async move {
            if let Err(e) = client.join(channel.clone()) {
                eprintln!("Failed to join Twitch channel: {:?}", e);
                return;
            }

            while let Some(message) = incoming_messages.recv().await {
                if let ServerMessage::Privmsg(msg) = message {
                    let name_color = msg.name_color.unwrap_or(RGBColor {
                        r: 255,
                        g: 255,
                        b: 255,
                    });
                    let color = Color::Rgb(name_color.r, name_color.g, name_color.b);

                    let chat_message = TwitchMessage::new(color, msg.sender.name, msg.message_text);

                    tx.send(chat_message).await.unwrap();
                }
            }
        });

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub struct TwitchMessage {
    pub sender_color: Color,
    pub sender: String,
    pub message: String,
}

impl TwitchMessage {
    fn new(sender_color: Color, sender: String, message: String) -> Self {
        Self {
            sender_color,
            sender,
            message,
        }
    }

    fn to_wrapped(&self, max_width: usize) -> Text<'_> {
        let full_message = format!("{}: {}", self.sender, self.message);
        let wrapped_lines = textwrap::wrap(full_message.as_str(), max_width);

        let mut lines = Vec::new();
        let mut sender_chars_left = self.sender.len();

        for wrapped_line in wrapped_lines.iter() {
            let mut spans = Vec::new();
            let mut current_idx = 0;

            if sender_chars_left > 0 {
                let chars_to_color = sender_chars_left.min(wrapped_line.len());
                let sender_part = &wrapped_line[0..chars_to_color];
                spans.push(Span::styled(
                    sender_part.to_string(),
                    Style::default().fg(self.sender_color),
                ));

                sender_chars_left -= chars_to_color;
                current_idx += chars_to_color;

                if sender_chars_left == 0 && current_idx < wrapped_line.len() {
                    spans.push(Span::raw(": ".to_string()));
                    current_idx += 2;
                }
            }

            if current_idx < wrapped_line.len() {
                let remaining_part = &wrapped_line[current_idx..];
                spans.push(Span::raw(remaining_part.to_string()));
            }

            lines.push(Line::from(spans));
        }

        Text::from(lines)
    }
}

pub struct TwitchChat {
    accent_color: Color,
    max_height: Arc<Mutex<usize>>,
    channel_name: String,
    twitch_client: TwitchClient,
    messages: Arc<Mutex<Vec<TwitchMessage>>>,
    rx: mpsc::Receiver<TwitchMessage>,
}

impl TwitchChat {
    pub fn new(accent_color: Color, channel_name: String) -> Self {
        let (tx, rx) = mpsc::channel(100);
        Self {
            accent_color,
            max_height: Arc::new(Mutex::new(0)),
            channel_name,
            twitch_client: TwitchClient::new(tx),
            messages: Arc::new(Mutex::new(Vec::new())),
            rx,
        }
    }

    pub fn start(&mut self) -> io::Result<()> {
        self.twitch_client.start(self.channel_name.clone())
    }

    pub fn poll_messages(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            let mut messages = self.messages.lock().unwrap();

            messages.push(message);
        }
    }
}

impl Widget for &TwitchChat {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let title_name = Line::from(Span::styled(
            format!(" {} ", self.channel_name),
            Style::default()
                .fg(self.accent_color)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center);

        let title_text = Line::from(Span::styled(
            " chat ",
            Style::new()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center);

        let chat_display = Block::default()
            .title_top(title_name)
            .title_bottom(title_text)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1));

        let mut messages = self.messages.lock().unwrap();

        let messages_area = chat_display.inner(area);

        let mut max_height_lock = self.max_height.lock().unwrap();
        if messages_area.height as usize > *max_height_lock {
            *max_height_lock = messages_area.height as usize;
        }

        let display_message_limit = *max_height_lock;

        if messages.len() > display_message_limit {
            let excess = messages.len() - display_message_limit;
            messages.drain(0..excess);
        }

        let texts: Vec<Text> = messages
            .iter()
            .rev()
            .map(|message| message.to_wrapped(messages_area.width as usize))
            .collect();

        let list = List::new(texts).direction(ListDirection::BottomToTop);

        chat_display.render(area, buf);

        list.render(messages_area, buf);
    }
}
