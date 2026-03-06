# Loom

A fast, privacy-focused CLI tool for recording browser windows and generating LLM-optimized videos. Built in Rust for macOS.

![Rust](https://img.shields.io/badge/rust-1.70%2B-orange)
![macOS](https://img.shields.io/badge/platform-macOS-blue)
![License](https://img.shields.io/badge/license-MIT-green)

## Why Loom?

When working with AI assistants like Claude, you often need to show them UI/UX changes. Traditional screen recording tools create large files that hit upload limits. Loom solves this by:

- **Targeting specific windows** - Record only what you need, no desktop clutter
- **Optimizing for LLMs** - Auto-compresses to H.264/720p, targeting <10MB per minute
- **Instant workflow** - Path copied to clipboard, LLM-ready prompt generated
- **Privacy-first** - Everything stays local, no cloud uploads, no accounts

## Features

- 🎯 **Smart Window Targeting** - Select any window by title
- 🎬 **H.264 Encoding** - Efficient MP4 output optimized for LLM visual analysis
- 📋 **Clipboard Integration** - File path automatically copied after recording
- 💬 **LLM Prompt Generation** - Pre-formatted prompt ready to paste
- ⏱️ **Duration Control** - Auto-stop or manual Ctrl+C
- 🔒 **100% Local** - No data leaves your machine

## Requirements

- **macOS** 12.0 or later (Monterey+)
- **ffmpeg** - For video encoding
- **Screen Recording permission** - Granted to your terminal

## Installation

### From Source

```bash
# Clone the repository
git clone https://github.com/yourusername/loom.git
cd loom

# Build release binary
cargo build --release

# Install to PATH
cargo install --path .
```

### Install ffmpeg

```bash
# Using Homebrew
brew install ffmpeg
```

## Usage

### List Available Windows

```bash
loom --list
```

Output:
```
Available windows:

  [1] Google Chrome - GitHub
  [2] VS Code - main.rs
  [3] Slack - general

Use --target "<title>" to select a specific window.
```

### Record a Window

```bash
# Interactive selection (prompts if multiple windows match)
loom

# Target specific window by title
loom --target "GitHub"

# Auto-stop after 30 seconds
loom --target "My App" --duration 30
```

### All Options

```
loom [OPTIONS]

Options:
  -t, --target <TARGET>      Window title filter (partial match)
  -d, --duration <SECONDS>   Auto-stop recording after N seconds
  -o, --output <PATH>        Output file path (default: ~/loom-recordings/)
      --fps <FPS>            Frame rate (default: 30)
      --resolution <WxH>     Target resolution (default: 1280x720)
  -l, --list                 List available windows and exit
  -v, --verbose              Enable debug logging
  -h, --help                 Print help
  -V, --version              Print version
```

## Workflow Example

```bash
# 1. List windows to find your target
loom --list

# 2. Start recording (Ctrl+C to stop)
loom --target "localhost:3000" --duration 60

# 3. Output:
Recording: localhost:3000 - My App
Press Ctrl+C to stop.
Recording... 45s elapsed
^C
Finalizing video...

Recording saved: /Users/you/loom-recordings/loom-2024-03-05_14-30-22.mp4
Duration: 45.2s
Size: 4.21 MB
Path copied to clipboard!

LLM Prompt:
────────────────────────────────────────
I recorded a video of a feature implementation. Please analyze this recording:

Video path: /Users/you/loom-recordings/loom-2024-03-05_14-30-22.mp4

The video shows my web application running in Chrome. Please:
1. Describe what you observe in the UI
2. Identify any potential UX improvements
3. Note any bugs or unexpected behavior

[Attach the video file to your message]
────────────────────────────────────────
```

## How It Works

```
┌─────────────────┐     ┌─────────────────┐     ┌─────────────────┐
│   Window        │     │   Frame         │     │   FFmpeg        │
│   Enumeration   │────▶│   Capture       │────▶│   Encoding      │
│   (scap)        │     │   (30 FPS)      │     │   (H.264)       │
└─────────────────┘     └─────────────────┘     └─────────────────┘
                                                        │
                                                        ▼
                        ┌─────────────────────────────────────┐
                        │   Output                            │
                        │   - ~/loom-recordings/loom-*.mp4    │
                        │   - Clipboard: file path            │
                        │   - LLM prompt template             │
                        └─────────────────────────────────────┘
```

### Architecture

| Component | Technology | Purpose |
|-----------|------------|---------|
| CLI | `clap` | Argument parsing |
| Capture | `scap` | Native macOS ScreenCaptureKit |
| Encoding | `ffmpeg` subprocess | H.264 video compression |
| Clipboard | `arboard` | Cross-platform clipboard |

## Permissions

Loom requires **Screen Recording** permission on macOS:

1. Open **System Settings** → **Privacy & Security** → **Screen Recording**
2. Enable your terminal app (Terminal.app, iTerm2, etc.)
3. Restart your terminal

If permission is not granted, Loom will prompt you and offer to open System Settings.

## Output Format

| Setting | Value | Reason |
|---------|-------|--------|
| Codec | H.264 | Universal compatibility |
| Resolution | 1280x720 | LLM-friendly, <10MB/min |
| Frame Rate | 30 FPS | Smooth motion |
| CRF | 23 | Quality/size balance |
| Container | MP4 + faststart | Web-ready streaming |

## Roadmap

- [ ] Windows support (Desktop Duplication API)
- [ ] Linux support (PipeWire)
- [ ] Audio capture
- [ ] GIF output option
- [ ] Config file (~/.loom.toml)
- [ ] Self-update command
- [ ] Homebrew formula

## Contributing

Contributions are welcome! Please feel free to submit a Pull Request.

### Development Setup

```bash
# Clone and build
git clone https://github.com/yourusername/loom.git
cd loom
cargo build

# Run tests
cargo test

# Run with debug logging
cargo run -- --verbose --list
```

### Code Structure

```
src/
├── main.rs           # Entry point, recording workflow
├── cli.rs            # CLI argument definitions
├── error.rs          # Custom error types
├── capture/
│   ├── mod.rs
│   ├── window.rs     # Window enumeration & selection
│   └── recorder.rs   # Frame capture pipeline
├── encode/
│   ├── mod.rs
│   └── ffmpeg.rs     # FFmpeg subprocess management
└── output/
    ├── mod.rs
    └── clipboard.rs  # Clipboard & prompt generation
```

## License

MIT License - see [LICENSE](LICENSE) for details.

## Acknowledgments

- [scap](https://github.com/clearlysid/scap) - Screen capture library
- [clap](https://github.com/clap-rs/clap) - CLI framework
- [ffmpeg](https://ffmpeg.org/) - Video encoding

