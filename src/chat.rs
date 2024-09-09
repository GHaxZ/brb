use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    text::{Span, Text},
    widgets::{Paragraph, Widget, Wrap},
};
use std::sync::{Arc, Mutex};
use twitch_irc::{
    login::StaticLoginCredentials, message::ServerMessage, ClientConfig, SecureTCPTransport,
    TwitchIRCClient,
};

pub struct ChatMessage {
    pub sender_color: Color,
    pub sender: String,
    pub message: String,
}

impl ChatMessage {
    fn new(sender_color: Color, sender: String, message: String) -> Self {
        Self {
            sender_color,
            sender,
            message,
        }
    }
}

impl ChatMessage {
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

pub struct Chat {
    twitch_channel: String,
    messages: Arc<Mutex<Vec<ChatMessage>>>,
}

impl Chat {
    pub fn new(twitch_channel: String) -> Self {
        Self {
            twitch_channel,
            messages: Arc::new(Mutex::new(Vec::new())),
        }
    }

    pub async fn run(&self) {
        let config = ClientConfig::default();
        let (mut incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        let messages = Arc::clone(&self.messages);

        // Spawn an async task to handle incoming IRC messages
        tokio::spawn(async move {
            while let Some(message) = incoming_messages.recv().await {
                match message {
                    ServerMessage::Privmsg(msg) => {
                        let color = match msg.name_color {
                            Some(c) => Color::Rgb(c.r, c.g, c.b),
                            None => Color::White,
                        };

                        let chat_message =
                            ChatMessage::new(color, msg.sender.name, msg.message_text);

                        let mut messages = messages.lock().unwrap();
                        messages.push(chat_message);
                        if messages.len() > 100 {
                            // Limit size to avoid memory issues
                            messages.remove(0);
                        }
                    }
                    _ => {}
                }
            }
        });

        // Join the Twitch channel
        if let Err(e) = client.join(self.twitch_channel.clone()) {
            eprintln!("Failed to join Twitch channel: {:?}", e);
        }
    }
}

impl Widget for Chat {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Lock the messages to read them
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
