# Smelter Crash

A minimal [Live Compositor](https://github.com/software-mansion/live-compositor) / Smelter project that records a web page to an MP4 file.

## Features

- Captures web pages using Chromium embedding
- Records to H.264 MP4 format (1920x1080 @ 30fps)
- Configurable recording duration via command line

## Prerequisites

- Rust toolchain
- FFmpeg libraries
- Chromium dependencies (installed automatically via compositor, only needed with `--web` flag)

## Building

```bash
cargo build --release
```

## Usage

```bash
cargo run -- [--web] [--ram <size>] [--rec <duration>] <duration>
```

### Arguments

- `<duration>` - Duration to run (required)
- `--web` - Use web renderer instead of MP4 input (optional, default: MP4)
- `--ram <size>` - Allocate memory before starting (optional, e.g., 100M, 2G)
- `--rec <duration>` - Record to MP4 file for this duration (optional, default: raw output only)

**Note:** By default, the program uses the MP4 file in `assets/test.mp4` as input. Use `--web` flag to render a web page instead. Without `--rec`, frames are generated and consumed but not saved to disk.

### Duration Format

The duration argument supports flexible time formats:

- `s` - seconds
- `m` - minutes
- `h` - hours

You can combine multiple units:

### Examples

```bash
# Run with raw output for 5 seconds (no recording)
cargo run -- 5s

# Record MP4 for 5 seconds
cargo run -- --rec 5s 5s

# Record MP4 for 10 minutes
cargo run -- --rec 10m 10m

# Run web page for 5 seconds (raw output, no recording)
cargo run -- --web 5s

# Record web page to MP4 for 5 seconds
cargo run -- --web --rec 5s 5s

# Record web page for 6 hours and 30 minutes
cargo run -- --web --rec 6h30m 6h30m

# Allocate 500MB RAM and run MP4 input for 5 seconds (raw output)
cargo run -- --ram 500M 5s

# Allocate 2GB RAM and record web page for 1 hour
cargo run -- --ram 2G --web --rec 1h 1h
```

## Configuration

You can modify the following constants in `src/main.rs`:

- `WIDTH` / `HEIGHT` - Output video resolution (default: 1920x1080)
- `WEB_URL` - The web page to capture when using `--web` (default: https://google.com)
- `MP4_INPUT` - Input MP4 file in assets folder (default: test.mp4)
- `OUTPUT_VIDEO` - Output filename (default: output.mp4)

## Output

When using `--rec`, the program creates `output.mp4` in the current directory. If the file already exists, it will be deleted before recording starts.

Without `--rec`, the program runs in raw output mode - frames are rendered and consumed through a channel but not saved to disk. This is useful for testing rendering performance without the overhead of encoding and writing to disk.

## Project Structure

This is a minimal example that demonstrates:

1. Initializing the compositor pipeline
2. Registering inputs (MP4 file or web renderer)
3. Creating a scene with the input
4. Recording the output to an MP4 file
5. Running the CEF event loop (required for web rendering)

No windowing, no input switching - just a straightforward input → MP4 pipeline.

### Key Implementation Details

- **Default mode (MP4)**: Simple video input stream, no event loop needed
- **Web mode (`--web`)**: Requires the CEF/Chromium event loop to run on the main thread for browser rendering to work
