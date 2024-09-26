/*
*   TODO:
*   Improve error handling for the twitch client
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

// A twitch client which can connect to a chat
pub struct TwitchClient {
    runtime: Runtime,                // The runtime used for receiving twitch messages
    tx: mpsc::Sender<TwitchMessage>, // The sender used for sending back new messages
}

impl TwitchClient {
    // Create a new client where the provided Sender is used to send back new messages
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

    // Start this twitch client on the provided channel
    pub fn start(&mut self, channel: String) -> io::Result<()> {
        let tx = self.tx.clone();
        // Create a default twitch client config
        let config = ClientConfig::default();
        // Here we log into the twitch API anonymously
        let (mut incoming_messages, client) =
            TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

        // Spawn a new task on the runtime
        self.runtime.spawn(async move {
            // Join the provided channel
            if let Err(e) = client.join(channel.clone()) {
                eprintln!("Failed to join Twitch channel: {:?}", e);
                return;
            }

            // While there are messages incoming
            while let Some(message) = incoming_messages.recv().await {
                // If we receive a "Privmsg", which is just a normal chat message
                if let ServerMessage::Privmsg(msg) = message {
                    // Turn the senders username color into a Color
                    let name_color = msg.name_color.unwrap_or(RGBColor {
                        r: 255,
                        g: 255,
                        b: 255,
                    });
                    let color = Color::Rgb(name_color.r, name_color.g, name_color.b);

                    // Create the message and send it into the channel
                    let chat_message = TwitchMessage::new(color, msg.sender.name, msg.message_text);
                    tx.send(chat_message).await.unwrap();
                }
            }
        });

        Ok(())
    }
}

// A twitch message received by the TwitchClient
#[derive(Clone, Debug)]
pub struct TwitchMessage {
    pub sender_color: Color, // The color of the message senders name
    pub sender: String,      // The name of the message sender
    pub message: String,     // The actual message content
}

impl TwitchMessage {
    fn new(sender_color: Color, sender: String, message: String) -> Self {
        Self {
            sender_color,
            sender,
            message,
        }
    }

    // I don't really have any details on this monstrosity, but I do know that it successfully
    // wraps lines, so don't touch it
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

// This is the actual TwitchChat widget which is displayed in the UI
pub struct TwitchChat {
    accent_color: Color,                      // Accent color which should be used
    max_height: Arc<Mutex<usize>>,            // Historical maximum chat area height
    channel_name: String,                     // Name of the chats twitch channel
    twitch_client: TwitchClient,              // TwitchClient used for receiving messages
    messages: Arc<Mutex<Vec<TwitchMessage>>>, // All currently stored messages
    rx: mpsc::Receiver<TwitchMessage>,        // Receiver for getting messages from TwitchClient
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

    // Start receiving messages for this TwitchChat
    pub fn start(&mut self) -> io::Result<()> {
        self.twitch_client.start(self.channel_name.clone())
    }

    // Poll for new messages
    pub fn poll_messages(&mut self) {
        while let Ok(message) = self.rx.try_recv() {
            let mut messages = self.messages.lock().unwrap();

            messages.push(message);
        }
    }
}

// Implement Widget for the TwitchChat so it can be rendered
impl Widget for &TwitchChat {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // The channel name text at the top
        let title_name = Line::from(Span::styled(
            format!(" {} ", self.channel_name),
            Style::default()
                .fg(self.accent_color)
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center);

        // The "chat" text at the bottom
        let title_text = Line::from(Span::styled(
            " chat ",
            Style::new()
                .add_modifier(Modifier::BOLD)
                .add_modifier(Modifier::ITALIC),
        ))
        .alignment(Alignment::Center);

        // Border of the chat
        let chat_display = Block::default()
            .title_top(title_name)
            .title_bottom(title_text)
            .border_type(BorderType::Rounded)
            .borders(Borders::ALL)
            .padding(Padding::horizontal(1));

        // Lock message vector
        let mut messages = self.messages.lock().unwrap();

        // Get a messages area inside the chat border
        let messages_area = chat_display.inner(area);

        // Lock the max height
        let mut max_height_lock = self.max_height.lock().unwrap();

        // If the message_area height is bigger than the current max
        if messages_area.height as usize > *max_height_lock {
            // Update the max value
            *max_height_lock = messages_area.height as usize;
        }

        // The stored messages limit is the same as the max height
        let display_message_limit = *max_height_lock;

        // If there are any excess messages
        if messages.len() > display_message_limit {
            // Remove them to free up memory
            let excess = messages.len() - display_message_limit;
            messages.drain(0..excess);
        }

        // Build the Text widgets out of the chat messages
        let texts: Vec<Text> = messages
            .iter()
            .rev()
            .map(|message| message.to_wrapped(messages_area.width as usize))
            .collect();

        // Create a new List for the chat messages and make it go bottom to top
        let list = List::new(texts).direction(ListDirection::BottomToTop);

        // And finally render the chat
        chat_display.render(area, buf);
        list.render(messages_area, buf);
    }
}
