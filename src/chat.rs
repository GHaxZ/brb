use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Paragraph, Widget, Wrap},
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
                match message {
                    ServerMessage::Privmsg(msg) => {
                        let name_color = msg.name_color.unwrap_or(RGBColor {
                            r: 255,
                            g: 255,
                            b: 255,
                        });
                        let color = Color::Rgb(name_color.r, name_color.g, name_color.b);

                        let chat_message =
                            TwitchMessage::new(color, msg.sender.name, msg.message_text);

                        // Send the message to the TUI via the channel
                        if tx.send(chat_message).await.is_err() {
                            eprintln!("Failed to send message to TUI");
                        }
                    }
                    _ => {}
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

    fn to_paragraph(&self) -> Paragraph<'_> {
        let sender = Span::styled(
            format!("{}: ", self.sender),
            Style::default().fg(self.sender_color),
        );
        let message = Span::raw(&self.message);

        let mut text = Text::from(sender);
        text.extend(Text::from(message));

        Paragraph::new(text).wrap(Wrap { trim: true })
    }
}

pub struct TwitchChat {
    twitch_client: TwitchClient,
    messages: Arc<Mutex<Vec<TwitchMessage>>>,
    rx: mpsc::Receiver<TwitchMessage>,
}

impl TwitchChat {
    pub fn new() -> Self {
        // Create an mpsc channel for communication between async Twitch client and TUI
        let (tx, rx) = mpsc::channel(100);
        Self {
            twitch_client: TwitchClient::new(tx),
            messages: Arc::new(Mutex::new(Vec::new())),
            rx,
        }
    }

    pub fn start(&mut self, channel_name: String) -> io::Result<()> {
        self.twitch_client.start(channel_name)
    }

    pub fn poll_messages(&mut self) {
        // Non-blocking read from the mpsc channel
        while let Ok(message) = self.rx.try_recv() {
            self.messages.lock().unwrap().push(message);
        }
    }
}

impl Widget for &TwitchChat {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let messages = self.messages.lock().unwrap();

        let layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints(
                messages
                    .iter()
                    .map(|_| Constraint::Min(1))
                    .collect::<Vec<Constraint>>(),
            )
            .split(area);

        for (i, message) in messages.iter().enumerate() {
            let paragraph = message.to_paragraph();
            paragraph.render(layout[i], buf);
        }
    }
}
