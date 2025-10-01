# Smelter Crash

Minimal [Live Compositor](https://github.com/software-mansion/live-compositor) example demonstrating video input (MP4/web) and output (MP4 recording/raw frames).

## Requirements
- Install Rust toolchain & FFmpeg libraries
- Build the process_helper: `cargo build --bin process_helper`

## Usage

```bash
cargo run -- [OPTIONS]
```

**Options:**
- `--rec <duration>` - Record to MP4, then exit (e.g., `5s`, `10m`, `1h30m`)
- `--web` - Use web renderer (default: MP4 from `assets/test.mp4`)
- `--ram <size>` - Allocate RAM before starting (e.g., `100M`, `2G`)
- `--help` - Show options and some examples

## Crash
- Does run: `cargo run -- --web`
- Crashes with "Illegal instruction" after a while: `cargo run -- --web --ram 3G`