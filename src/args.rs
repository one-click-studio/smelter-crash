use anyhow::{anyhow, Result};
use std::time::Duration;

#[derive(Debug)]
pub struct Args {
    pub use_web: bool,
    pub duration: Option<Duration>,
    pub allocate_ram: Option<String>,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();

        let mut use_web = false;
        let mut duration: Option<Duration> = None;
        let mut allocate_ram: Option<String> = None;

        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];
            if arg == "--help" || arg == "-h" {
                print_usage(&args[0]);
                std::process::exit(0);
            } else if arg == "--web" {
                use_web = true;
                i += 1;
            } else if arg == "--ram" {
                if i + 1 >= args.len() {
                    return Err(anyhow!("--ram requires a value (e.g., 100M, 2G)"));
                }
                allocate_ram = Some(args[i + 1].clone());
                i += 2;
            } else if arg == "--rec" {
                if i + 1 >= args.len() {
                    return Err(anyhow!("--rec requires a duration (e.g., 5s, 10m, 2h)"));
                }
                duration = Some(parse_duration(&args[i + 1])?);
                i += 2;
            } else {
                return Err(anyhow!("Unknown argument: {}", arg));
            }
        }

        Ok(Args {
            use_web,
            duration,
            allocate_ram,
        })
    }
}

fn print_usage(program_name: &str) {
    eprintln!("Usage: {} [OPTIONS]", program_name);
    eprintln!("");
    eprintln!("Options:");
    eprintln!("  --rec <duration>    Record to MP4 file for this duration (optional)");
    eprintln!("  --web               Use web renderer instead of MP4 input");
    eprintln!("  --ram <size>        Allocate memory before starting (e.g., 100M, 2G)");
    eprintln!("");
    eprintln!("Duration format: Xs (seconds), Xm (minutes), Xh (hours), or combinations like 1h30m");
    eprintln!("");
    eprintln!("Examples:");
    eprintln!("  {}                       - Run indefinitely with raw output (Ctrl+C to stop)", program_name);
    eprintln!("  {} --rec 5s              - Record MP4 for 5 seconds then run indefinitely", program_name);
    eprintln!("  {} --web --rec 10m       - Record web page to MP4 for 10 minutes then run indefinitely", program_name);
    eprintln!("  {} --ram 2G --web        - Allocate 2GB RAM and run web page indefinitely (raw output)", program_name);
    eprintln!("  {} --ram 500M --rec 30s  - Allocate 500MB RAM, record for 30 seconds, then run indefinitely", program_name);
}

fn parse_duration(input: &str) -> Result<Duration> {
    let input = input.trim();
    let mut total_secs = 0u64;
    let mut current_num = String::new();

    for ch in input.chars() {
        if ch.is_ascii_digit() {
            current_num.push(ch);
        } else if ch == 's' || ch == 'm' || ch == 'h' {
            if current_num.is_empty() {
                return Err(anyhow!("Invalid duration format: missing number before '{}'", ch));
            }
            let num: u64 = current_num.parse()
                .map_err(|_| anyhow!("Failed to parse number: {}", current_num))?;

            let multiplier = match ch {
                's' => 1,
                'm' => 60,
                'h' => 3600,
                _ => unreachable!(),
            };

            total_secs += num * multiplier;
            current_num.clear();
        } else if !ch.is_whitespace() {
            return Err(anyhow!("Invalid character '{}' in duration. Use only numbers and s/m/h", ch));
        }
    }

    if !current_num.is_empty() {
        return Err(anyhow!("Invalid duration format: trailing number without unit (s/m/h)"));
    }

    if total_secs == 0 {
        return Err(anyhow!("Duration must be greater than 0"));
    }

    Ok(Duration::from_secs(total_secs))
}
