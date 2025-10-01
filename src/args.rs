use anyhow::{anyhow, Result};

#[derive(Debug)]
pub struct Args {
    pub allocate_ram: Option<String>,
}

impl Args {
    pub fn parse() -> Result<Self> {
        let args: Vec<String> = std::env::args().collect();

        let mut allocate_ram: Option<String> = None;

        let mut i = 1;
        while i < args.len() {
            let arg = &args[i];
            if arg == "--ram" {
                if i + 1 >= args.len() {
                    return Err(anyhow!("--ram requires a value (e.g., 100M, 2G)"));
                }
                allocate_ram = Some(args[i + 1].clone());
                i += 2;
            } else {
                return Err(anyhow!("Unknown argument: {}", arg));
            }
        }

        Ok(Args { allocate_ram })
    }
}
