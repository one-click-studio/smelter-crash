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
cargo run -- [--web] <duration>
```

### Arguments

- `<duration>` - Duration to record (required)
- `--web` - Use web renderer instead of MP4 input (optional, default: MP4)

**Note:** By default, the program records from the MP4 file in `assets/test.mp4`. Use `--web` flag to capture a web page instead.

### Duration Format

The duration argument supports flexible time formats:

- `s` - seconds
- `m` - minutes
- `h` - hours

You can combine multiple units:

### Examples

```bash
# Record MP4 for 5 seconds (default)
cargo run -- 5s

# Record MP4 for 10 minutes
cargo run -- 10m

# Record web page for 5 seconds
cargo run -- --web 5s

# Record web page for 2 hours
cargo run -- --web 2h

# Record web page for 6 hours and 30 minutes
cargo run -- --web 6h30m
```

## Configuration

You can modify the following constants in `src/main.rs`:

- `WIDTH` / `HEIGHT` - Output video resolution (default: 1920x1080)
- `WEB_URL` - The web page to capture when using `--web` (default: https://google.com)
- `MP4_INPUT` - Input MP4 file in assets folder (default: test.mp4)
- `OUTPUT_VIDEO` - Output filename (default: output.mp4)

## Output

The program will create `output.mp4` in the current directory. If the file already exists, it will be deleted before recording starts.

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
