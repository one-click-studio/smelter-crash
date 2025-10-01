use anyhow::{anyhow, Result};
use std::thread;
use std::time::Duration;
use tracing::info;

pub fn allocate_and_hold(ram_size: String) -> Result<()> {
    let bytes = parse_memory_size(&ram_size)?;

    thread::spawn(move || {
        info!("Allocating {} of RAM...", ram_size);
        let mut memory: Vec<u8> = vec![0; bytes];

        // Force actual memory allocation by writing to every page (typically 4KB)
        let page_size = 4096;
        for i in (0..bytes).step_by(page_size) {
            memory[i] = 1;
        }

        info!("Allocated {} of RAM, holding indefinitely", ram_size);

        // Keep the memory allocated forever
        loop {
            thread::sleep(Duration::from_secs(3600));
        }
    });

    Ok(())
}

fn parse_memory_size(input: &str) -> Result<usize> {
    let input = input.trim().to_uppercase();

    // Find where the number ends and the unit begins
    let split_pos = input
        .chars()
        .position(|c| !c.is_ascii_digit())
        .unwrap_or(input.len());

    let (num_str, unit_str) = input.split_at(split_pos);

    if num_str.is_empty() {
        return Err(anyhow!("Invalid memory size format: missing number"));
    }

    let num: usize = num_str
        .parse()
        .map_err(|_| anyhow!("Failed to parse number: {}", num_str))?;

    let multiplier: usize = match unit_str.trim() {
        "" | "B" => 1,                          // Bytes
        "K" | "KB" => 1024,                     // Kilobytes
        "M" | "MB" => 1024 * 1024,              // Megabytes
        "G" | "GB" => 1024 * 1024 * 1024,       // Gigabytes
        _ => return Err(anyhow!("Invalid memory unit: '{}'. Use B, K/KB, M/MB, or G/GB", unit_str)),
    };

    num.checked_mul(multiplier)
        .ok_or_else(|| anyhow!("Memory size too large: {} would overflow", input))
}
