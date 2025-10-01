use std::thread;
use std::time::{Duration, Instant};
use tracing::{info, warn};

const MONITOR_INTERVAL_SECS: u64 = 10;

#[repr(C)]
#[derive(Debug, Clone, Copy)]
struct MallInfo {
    arena: i32,
    ordblks: i32,
    smblks: i32,
    hblks: i32,
    hblkhd: i32,
    usmblks: i32,
    fsmblks: i32,
    uordblks: i32,
    fordblks: i32,
    keepcost: i32,
}

extern "C" {
    fn mallinfo() -> MallInfo;
}

#[derive(Debug, Clone, Copy)]
struct MallinfoSnapshot {
    info: MallInfo,
}

impl MallinfoSnapshot {
    fn new(info: MallInfo) -> Self {
        Self { info }
    }

    /// Check if the mallinfo for invalid values
    fn check_for_wraparound(&self) -> Vec<String> {
        let mut warnings = Vec::new();

        if self.info.arena < 0 {
            warnings.push(format!("arena is negative: {} (integer overflow!)", self.info.arena));
        }
        if self.info.uordblks < 0 {
            warnings.push(format!("uordblks is negative: {} (integer overflow!)", self.info.uordblks));
        }
        if self.info.fordblks < 0 {
            warnings.push(format!("fordblks is negative: {} (integer overflow!)", self.info.fordblks));
        }
        if self.info.hblkhd < 0 {
            warnings.push(format!("hblkhd is negative: {} (integer overflow!)", self.info.hblkhd));
        }

        let arena_plus_hblkhd = self.info.arena as i64 + self.info.hblkhd as i64;
        if arena_plus_hblkhd > i32::MAX as i64 {
            warnings.push(format!(
                "arena + hblkhd > INT_MAX ({} + {} > {})",
                self.info.arena, self.info.hblkhd, i32::MAX
            ));
        }

        if self.info.uordblks > i32::MAX {
            warnings.push(format!(
                "uordblks > INT_MAX ({} > {})",
                self.info.uordblks, i32::MAX
            ));
        }

        warnings
    }

    /// Format the mallinfo data in a human-readable way
    fn format_readable(&self) -> String {
        format!(
            "arena: {} ({:.2} MB)\n  ordblks: {}\n  hblks: {}\n  hblkhd: {} ({:.2} MB)\n  uordblks: {} ({:.2} MB)\n  fordblks: {} ({:.2} MB)\n  keepcost: {} ({:.2} MB)",
            self.info.arena,
            self.info.arena as f64 / 1_048_576.0,
            self.info.ordblks,
            self.info.hblks,
            self.info.hblkhd,
            self.info.hblkhd as f64 / 1_048_576.0,
            self.info.uordblks,
            self.info.uordblks as f64 / 1_048_576.0,
            self.info.fordblks,
            self.info.fordblks as f64 / 1_048_576.0,
            self.info.keepcost,
            self.info.keepcost as f64 / 1_048_576.0,
        )
    }
}

/// Starts a background thread that reports mallinfo statistics every 10 seconds
pub fn start_memory_monitor() {
    thread::spawn(|| {
        thread::sleep(Duration::from_secs(1));
        let start_time = Instant::now();
        info!("Memory monitor started - will report mallinfo() every {} seconds", MONITOR_INTERVAL_SECS);

        loop {
            let info = unsafe { mallinfo() };
            let snapshot = MallinfoSnapshot::new(info);

            let elapsed = start_time.elapsed().as_secs();
            info!("Mallinfo (elapsed time: {}s):\n  {}", elapsed, snapshot.format_readable());
            let warnings = snapshot.check_for_wraparound();
            for warning in warnings {
                warn!("{}", warning);
            }

            thread::sleep(Duration::from_secs(MONITOR_INTERVAL_SECS));
        }
    });
}
