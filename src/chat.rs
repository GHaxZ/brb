/*
*   TODO:
*   Make the chat messages appear from bottom
*   Add padding between messages
*/

use ratatui::{
    buffer::Buffer,
    layout::{Alignment, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Borders, Padding, Paragraph, Widget, Wrap},
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

    fn to_line(&self) -> Line<'_> {
        let sender = Span::styled(
            format!("{}: ", self.sender),
            Style::default().fg(self.sender_color),
        );

        let message = Span::raw(&self.message);

        Line::from(vec![sender, message])
    }
}

pub struct TwitchChat {
    accent_color: Color,
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

        let mut text_buffer = Vec::new();

        let messages = self.messages.lock().unwrap();

        for message in messages.iter() {
            let message_text = message.to_line();
            text_buffer.push(message_text);
        }

        let paragraph = Paragraph::new(text_buffer)
            .block(chat_display)
            .wrap(Wrap { trim: false });

        paragraph.render(area, buf);
    }
}
