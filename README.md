# Arctic Lock ❄️

A high-performance, aesthetically pleasing X11 screen locker written in Rust.

Arctic Lock is designed to be lightweight, secure, and visually polished. It renders a synchronized lock screen across all connected monitors, supports custom backgrounds, and features the "Arctic Abyssal" coding theme.

## ✨ Features

- **Multi-Monitor Support**: Automatically detects all connected screens and renders the lock UI centered on each one independently.
- **Security First**:
  - Bypasses window managers using `override_redirect`.
  - Robust input grabbing loop to prevent focus stealing.
  - Uses system PAM (Pluggable Authentication Modules) for secure authentication.
- **Aesthetics**:
  - **Arctic Abyssal Theme**: Deep blue/teal color palette optimized for developers.
  - **Animations**: Shake-on-error and blinking cursor.
  - **Typography**: Clean rendering using any TTF font.
  - **Custom Wallpaper**: Loads and scales your preferred background image.
- **Dev-Friendly**:
  - Displays active user and current time/date.
  - Shows random "dev excuses" (funny coding phrases) upon failed login attempts.

## 🛠️ Prerequisites

To build Arctic Lock, you need the Rust toolchain and the development headers for X11 and PAM.

### Debian / Ubuntu

```bash
sudo apt update
sudo apt install cargo libpam0g-dev libx11-dev libxrandr-dev
```

### Arch Linux

```bash
sudo pacman -S rust cargo pam libx11 libxrandr
```

## 📦 Installation

1. **Clone the repository:**

```bash
git clone <your-repo-url>
cd arctic-lock
```

2. **Build the release binary:**

```bash
cargo build --release
```

3. **Install system-wide:**

> **Note:** Arctic Lock requires root ownership and the setuid bit to read `/etc/shadow` for password verification.

```bash
# Move binary to local bin
sudo mv target/release/arctic-lock /usr/local/bin/

# Set ownership to root
sudo chown root:root /usr/local/bin/arctic-lock

# Set SUID permissions (Essential!)
sudo chmod 4755 /usr/local/bin/arctic-lock
```

## 🚀 Usage

You must provide a path to a TrueType Font (`.ttf`) file. Optionally, you can provide a background image.

### Syntax

```bash
arctic-lock <path_to_font.ttf> [path_to_background.png]
```

### Example

```bash
# Minimal (Black background)
arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf

# With Custom Wallpaper
arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf ~/Pictures/wallpapers/ocean.png
```

### Tips

- **Keyboard Shortcut**: Bind the command above to `Super+L` or `Ctrl+Alt+L` in your window manager (i3, bspwm, AwesomeWM, etc.) config.
- **Fonts**: Most Linux distributions store fonts in `/usr/share/fonts`. `DejaVuSans.ttf` or `Hack.ttf` are good choices.

## ⌨️ Controls

- **Type Password**: Input your login password.
- **Enter**: Submit password.
- **Backspace**: Delete last character.
- **Escape**: Clear the entire password field.

## 🔧 Troubleshooting

**"MaximumRequestLengthExceeded" panic**: This version includes chunked image rendering to prevent this X11 error. If you still encounter display issues on 4K+ screens, ensure you are using the latest version of the code.

**"Authentication Failed" (even with correct password)**: Double-check permissions. Run:

```bash
ls -l /usr/local/bin/arctic-lock
```

Output must look like: `-rwsr-xr-x 1 root root ....` If the `s` is missing, run the `chmod 4755` command again.
