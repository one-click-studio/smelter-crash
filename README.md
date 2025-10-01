# Smelter Crash

Minimal [Live Compositor](https://github.com/software-mansion/live-compositor) example demonstrating web input and raw output.

## Requirements
- Install Rust toolchain & FFmpeg libraries
- Build the process_helper: `cargo build --bin process_helper`

## Usage

```bash
cargo run -- [OPTIONS]
```

**Options:**
- `--ram <size>` - Allocate RAM before starting (e.g., `100M`, `2G`)

## Crash
- Does run: `cargo run`
- Crashes with "Illegal instruction" after a while: `cargo run -- --ram 2000MB`

