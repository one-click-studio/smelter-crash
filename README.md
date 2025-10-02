# Smelter crash

Minimal [Smelter](https://github.com/software-mansion/live-compositor) example to show how it can randomly crash on Linux when the RAM usage is around 2 GB.
See the detailed explanation [here](./CRASH_EXPLAINATION.md).

## Requirements

- Build the process_helper: `cargo build --bin process_helper`
- Build the patch: `cargo build -p mallinfo-override`

## Usage

### Crash
This project allows to allocate the right amount of RAM to trigger the bug. Run the test with:
```sh
cargo run -- --ram 2000MB
```

You should start to have these warnings:
```sh
WARN smelter_crash::memory_monitor: arena + hblkhd > INT_MAX (176197632 + 2097156096 > 2147483647)
```
If not, check the Mallinfo logs while it runs to get the `arena` value. And change the RAM parameter that will influence `hblkhd`.

It should crash after a while with **Illegal instruction**.
From our experience this usually takes from 10 to 60 minutes, but can take up to 2 hours.

### Patch
This command demonstrates how overriding mallinfo prevents the crash:
```sh
LD_PRELOAD=target/debug/libmallinfo_override.so cargo run -- --ram 2000MB
```
