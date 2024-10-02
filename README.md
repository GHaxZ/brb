# brb 🏃

#### For when you'll be right back!

![demo.gif](https://github.com/GHaxZ/brb/blob/master/.github/demo.gif)

"**brb**" is a **terminal live streaming tool** that lets your viewers know that you are currently away in a unique way!

## Features

- **Countdown**
  - Your viewers know when you'll be back!
- **Live Twitch chat**
  - Displays Twitch name colors
  - No authentication is required
- **Configurability**
  - Hide elements
  - Change text
  - Change colors
- **Lightweight**
  - ~6MB RAM no chat, ~10MB with chat

## Installation

Head to the [releases](https://github.com/GHaxZ/brb/releases) page and search for the latest release.

You are presented with multiple ways to install brb:
- **Shell script**
  - Useable on Linux and macOS
  - No extra software required
  - No automatic updates
- **Powershell script**
  - Usable on Windows
  - No extra software required
  - No automatic updates
- **Homebrew**
  - Usable on Linux and macOS
  - Extra software required ([homebrew](https://brew.sh/))
  - Automatic updates

Alternatively, if you have Rust installed, you can compile yourself:
```bash
cargo install --git https://github.com/GHaxZ/brb.git
```

## Usage

### Command usage

To **run brb** just enter, well:

```bash
brb
```

**Set a countdown** by entering an amount of hours, minutes, or seconds:

```bash
brb 1h 23m 45s
```

To **set a text** use:

```bash
brb -t "Hello world!"
```

To **see all available commands**, you can run:

```bash
brb -h
```

### Configuration file

Using the right arguments every time is annoying, so instead, you can use a configuration file.

**Depending on your OS, put the configuration file in**:

- **Linux**: ~/.config/brb/brb.toml
- **Windows**: %APPDATA%\brb\brb.toml
- **macOS**: ~/Library/Application Support/brb/brb.toml

#### Example config:

```toml
# Set custom color

# Either choose from black, red, green, yellow, blue, magenta, cyan, or white:
color = "red"

# Or define a custom RGB color:
# color = { r = 95, g = 126, b = 255 }

# Set the Twitch channel name for the chat
twitch_channel = "ghax_z"

# Set the text in the center
text = "Be right back"

# Enable/disable the chat
chat = true

# Hide the timer after the time is up
hide_timer = true

# Enable/disable the progress bar
progress_bar = true
```

## Contributing

Contributions are always welcome!

Please make sure you somewhat **adhere to the codebase style** and **document your code**, especially in hard-to-understand areas.

Thanks!

## Feedback

In case you encounter any **issues** or have a **feature you want to see**, please [open a github issue](https://github.com/GHaxZ/brb/issues/new). I'll do my best to fix things!

## License

This project is licensed under the [MIT](https://choosealicense.com/licenses/mit/) license.
