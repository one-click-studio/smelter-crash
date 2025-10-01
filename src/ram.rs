use anyhow::{anyhow, Result};
use tracing::info;

pub fn allocate(ram_size: &str) -> Result<Vec<u8>> {
    let bytes = parse_memory_size(ram_size)?;
    info!("Allocating {} bytes ({}) of RAM...", bytes, ram_size);
    let memory: Vec<u8> = vec![0; bytes];
    info!("Successfully allocated {} of RAM", ram_size);
    Ok(memory)
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
