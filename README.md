# Smelter Crash

A minimal [Live Compositor](https://github.com/software-mansion/live-compositor) / Smelter project that records a web page to an MP4 file.

## Features

- Captures web pages using Chromium embedding
- Records to H.264 MP4 format (1920x1080 @ 30fps)
- Configurable recording duration via command line

## Prerequisites

- Rust toolchain
- FFmpeg libraries
- Chromium dependencies (installed automatically via compositor)
- **xvfb** (X Virtual Frame Buffer) for headless Chrome rendering

## Building

```bash
cargo build --release
```

## Usage

**Important:** Web rendering requires a display. Use `xvfb-run` to provide a virtual display:

```bash
xvfb-run cargo run -- <duration>
```

Or install xvfb if not already installed:

```bash
# Ubuntu/Debian
sudo apt-get install xvfb

# Arch Linux
sudo pacman -S xorg-server-xvfb
```

### Duration Format

The duration argument supports flexible time formats:

- `s` - seconds
- `m` - minutes
- `h` - hours

You can combine multiple units:

### Examples

```bash
# Record for 5 seconds
xvfb-run cargo run -- 5s

# Record for 10 minutes
xvfb-run cargo run -- 10m

# Record for 2 hours
xvfb-run cargo run -- 2h

# Record for 6 hours and 30 minutes
xvfb-run cargo run -- 6h30m

# Record for 1 hour, 15 minutes, and 30 seconds
xvfb-run cargo run -- 1h15m30s
```

## Configuration

You can modify the following constants in `src/main.rs`:

- `WIDTH` / `HEIGHT` - Output video resolution (default: 1920x1080)
- `WEB_URL` - The web page to capture (default: https://google.com)
- `OUTPUT_VIDEO` - Output filename (default: output.mp4)

## Output

The program will create `output.mp4` in the current directory. If the file already exists, it will be deleted before recording starts.

## Project Structure

This is a minimal example that demonstrates:

1. Initializing the compositor pipeline with web renderer enabled
2. Registering a web renderer for a URL
3. Creating a scene with the web view
4. Recording the output to an MP4 file

No windowing, no input switching - just a straightforward web page → MP4 pipeline.
