# Arctic Lock ❄️

⚠️ **Educational Project - Use With Caution** ⚠️

A high-performance, aesthetically pleasing X11 screen locker written in Rust. This is a learning project exploring X11 screen locking mechanisms. While functional, this is **NOT recommended for security-critical environments**. For production use, consider mature alternatives like `i3lock`, `xsecurelock`, or `swaylock`.

Arctic Lock is designed to be lightweight, secure, and visually polished. It renders a synchronized lock screen across all connected monitors, supports custom backgrounds, and features the "Arctic Abyssal" coding theme.

---

## ⚠️ SECURITY NOTICE

**READ THIS BEFORE INSTALLING**

### Critical Security Information

- **SUID Root Binary**: This program requires root privileges to access PAM authentication. Any bugs in the code could potentially lead to privilege escalation.
- **Educational Purpose**: This is a learning project and proof of concept, not enterprise security software.
- **Use Only If**: You understand the security implications, have reviewed the source code, and accept the risks.
- **Not Recommended For**: Production environments, shared systems, multi-user environments, or high-security scenarios.

### Known Limitations

Please be aware of the following security considerations:

- ⚠️ Password stored in plain String (not cleared from memory)
- ⚠️ If input grab fails, program continues running (may appear locked when it's not)
- ⚠️ No rate limiting on authentication attempts
- ⚠️ No account lockout after failed attempts
- ⚠️ PAM initialization errors will panic (could unlock screen)
- ⚠️ No privilege dropping after initialization

**Recommendation**: For production use or security-critical environments, use the mature alternatives listed above.

### Mature Alternatives

For production use, consider these battle-tested alternatives:

- **i3lock** - Minimal, proven screen locker
- **xsecurelock** - Extensive security features
- **swaylock** - For Wayland compositors
- **slock** - Suckless simple locker

### Reporting Security Issues

If you discover a security vulnerability, please report it privately to [your-email@example.com] before public disclosure. Please do not open public GitHub issues for security problems.

---

## ✨ Features

- **Multi-Monitor Support**: Automatically detects all connected screens and renders the lock UI centered on each one independently.
- **Security Features**:
  - Bypasses window managers using `override_redirect`
  - Input grabbing to prevent window switching
  - Uses system PAM (Pluggable Authentication Modules) for authentication
- **Aesthetics**:
  - **Arctic Abyssal Theme**: Deep blue/teal color palette optimized for developers
  - **Animations**: Shake-on-error and blinking cursor
  - **Typography**: Clean rendering using any TTF font
  - **Custom Wallpaper**: Loads and scales your preferred background image
- **Dev-Friendly**:
  - Displays active user and current time/date
  - Shows random "dev excuses" (funny coding phrases) upon failed login attempts

---

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

### Fedora / RHEL

```bash
sudo dnf install cargo pam-devel libX11-devel libXrandr-devel
```

---

## 📦 Installation

### 1. Clone the repository

```bash
git clone https://github.com/yourusername/arctic-lock.git
cd arctic-lock
```

### 2. Review the source code

**IMPORTANT**: Before building and installing any SUID binary, you should review the source code to ensure you understand what it does and trust it.

```bash
# Review the main source file
cat src/main.rs

# Check dependencies
cat Cargo.toml
```

### 3. Build the release binary

```bash
cargo build --release
```

### 4. Install system-wide

> **⚠️ CRITICAL SECURITY WARNING**: Arctic Lock requires root ownership and the setuid bit to access `/etc/shadow` for password verification. This means any security vulnerability in the code could be exploited for privilege escalation. Only proceed if you understand and accept this risk.

```bash
# Move binary to local bin
sudo mv target/release/arctic-lock /usr/local/bin/

# Set ownership to root
sudo chown root:root /usr/local/bin/arctic-lock

# Set SUID permissions (Essential but risky - see warning above)
sudo chmod 4755 /usr/local/bin/arctic-lock
```

### 5. Verify installation

```bash
ls -l /usr/local/bin/arctic-lock
```

Output should look like: `-rwsr-xr-x 1 root root ...` (note the `s` in permissions)

---

## 🚀 Usage

You must provide a path to a TrueType Font (`.ttf`) file. Optionally, you can provide a background image.

### Syntax

```bash
arctic-lock <path_to_font.ttf> [path_to_background.png]
```

### Examples

```bash
# Minimal (solid background)
arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf

# With Custom Wallpaper
arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf ~/Pictures/wallpapers/ocean.png

# Using Hack font
arctic-lock /usr/share/fonts/truetype/hack/Hack-Regular.ttf ~/wallpaper.jpg
```

### Tips

- **Keyboard Shortcut**: Bind the command above to `Super+L` or `Ctrl+Alt+L` in your window manager config:

  **i3 Config Example:**

  ```
  bindsym $mod+l exec --no-startup-id /usr/local/bin/arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf
  ```

  **bspwm Config Example:**

  ```
  super + l
      /usr/local/bin/arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf
  ```

- **Font Locations**: Most Linux distributions store fonts in `/usr/share/fonts`. Good choices include:
  - DejaVu: `/usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf`
  - Hack: `/usr/share/fonts/truetype/hack/Hack-Regular.ttf`
  - Liberation: `/usr/share/fonts/truetype/liberation/LiberationSans-Regular.ttf`

- **Find Fonts**: Use `fc-list` to find available fonts:

  ```bash
  fc-list | grep -i "dejavu\|hack\|liberation"
  ```

---

## ⌨️ Controls

- **Type Password**: Input your login password
- **Enter**: Submit password for authentication
- **Backspace**: Delete last character
- **Escape**: Clear the entire password field

---

## 🔧 Troubleshooting

### "MaximumRequestLengthExceeded" panic

**Solution**: The current version includes chunked image rendering to prevent this X11 error. If you still encounter display issues on 4K+ screens, ensure you are using the latest version of the code.

### "Authentication Failed" (even with correct password)

**Cause**: Incorrect permissions on the binary.

**Solution**: Double-check permissions:

```bash
ls -l /usr/local/bin/arctic-lock
```

Output must look like: `-rwsr-xr-x 1 root root ....`

If the `s` is missing, run:

```bash
sudo chmod 4755 /usr/local/bin/arctic-lock
```

### "CRITICAL ERROR: Failed to grab inputs"

**Cause**: Another application has grabbed the keyboard/pointer, or the window manager is preventing the grab.

**What it means**: The program continues running but may not have successfully grabbed input devices. **This is a security risk** - the screen may appear locked but inputs might not be captured.

**Solutions**:

- Close applications that might grab input (games, VMs, screen recorders)
- If using a compositor, try disabling it temporarily
- Check for other screen locking software running

### Screen doesn't lock all monitors

**Cause**: RandR monitor detection failed.

**Solution**:

1. Check monitor setup: `xrandr --listmonitors`
2. Ensure all monitors are properly configured
3. Try updating your graphics drivers

### PAM authentication errors in logs

**Cause**: PAM configuration issues.

**Solution**: Check PAM configuration:

```bash
cat /etc/pam.d/login
```

Ensure the file exists and is properly configured for your system.

---

## 🏗️ Building from Source

### Dependencies in Cargo.toml

```toml
[dependencies]
chrono = "0.4"
image = "0.24"
pam = "0.7"
rand = "0.8"
rusttype = "0.9"
users = "0.11"
x11rb = "0.13"
```

### Build Options

```bash
# Development build (with debug symbols)
cargo build

# Release build (optimized)
cargo build --release

# Run tests (if available)
cargo test

# Check code without building
cargo check
```

---

## 🧪 Testing Recommendations

Before relying on Arctic Lock, test it thoroughly:

1. **Test with wrong password**: Verify it denies access
2. **Test input grabbing**: Ensure you cannot switch to other windows (note: if grab fails, this may not work)
3. **Test multi-monitor**: Verify all screens are covered
4. **Test with background**: Try different image formats and sizes
5. **Test edge cases**: Try very long passwords, special characters, etc.

**Testing command** (use a throw-away terminal):

```bash
# This will lock your screen - have a backup way to unlock ready!
arctic-lock /usr/share/fonts/truetype/dejavu/DejaVuSans-Bold.ttf
```

---

## 🤝 Contributing

Contributions are welcome, especially:

- Security improvements and audits
- Bug fixes
- Documentation improvements
- Testing on different systems

### Guidelines

1. **Security First**: All PRs involving security should be thoroughly reviewed
2. **Test Thoroughly**: Ensure changes don't break existing functionality
3. **Document Changes**: Update README and code comments
4. **Follow Rust Best Practices**: Run `cargo clippy` and `cargo fmt`

### Reporting Issues

- **Security Issues**: Report privately (see Security Notice above)
- **Bugs**: Open a GitHub issue with details about your system and the problem
- **Feature Requests**: Open a GitHub issue with your proposal

---

## 📚 How It Works

### Architecture Overview

1. **X11 Connection**: Establishes connection to X server
2. **Monitor Detection**: Uses RandR extension to detect all monitors
3. **Window Creation**: Creates fullscreen override-redirect window
4. **Input Grabbing**: Grabs keyboard and pointer (CRITICAL for security)
5. **PAM Authentication**: Uses system PAM for password verification
6. **Rendering Loop**: 60 FPS rendering loop for animations
7. **Security Layer**: Rate limiting, lockout, and secure memory handling

### Security Model

- **SUID Root**: Required for PAM access to `/etc/shadow`
- **Input Isolation**: Attempts to grab all input to prevent interaction with other apps
- **Memory Safety**: Rust's ownership system prevents many common bugs

### Why Rust?

- Memory safety without garbage collection
- Strong type system prevents common bugs
- Zero-cost abstractions for performance
- Excellent X11 bindings via `x11rb`

---

## 📋 System Requirements

- **OS**: Linux with X11 (not Wayland)
- **Display Server**: X.Org
- **PAM**: System must have PAM configured
- **Rust**: 1.70.0 or newer (for building)

### Tested On

- Arch Linux (X11)
- Ubuntu 22.04 LTS (X11)
- Debian 12 (X11)
- Fedora 38 (X11)

**Note**: Does NOT work on Wayland. For Wayland, use `swaylock` instead.

---

## 🗺️ Roadmap

### Planned Features

- [ ] Wayland support (via wlroots)
- [ ] Configuration file support
- [ ] Theming system
- [ ] Plugin architecture
- [ ] Screen blanking after timeout
- [ ] Privilege dropping after PAM initialization
