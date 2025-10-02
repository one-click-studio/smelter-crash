#![cfg(target_os = "linux")]
#![cfg_attr(
    not(target_os = "linux"),
    allow(dead_code, unused_imports)
)]

/// mallinfo() Override for CEF - Prevents "Illegal instruction" crash
///
/// This shared library intercepts mallinfo() calls and returns safe values
/// using mallinfo2() on systems with glibc >= 2.33, ensuring that:
/// 1. No individual field is negative
/// 2. arena + hblkhd <= INT_MAX
/// 3. uordblks <= INT_MAX
///
/// NOTE: This library is Linux-only. macOS doesn't need it because CEF uses
/// malloc_zone_statistics() instead of mallinfo() on Apple platforms.

use std::sync::atomic::{AtomicBool, Ordering};
use tracing::{debug, warn};

/// C struct mallinfo layout (glibc)
/// signed 32-bit integers
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct MallinfoC {
    pub arena: i32,
    pub ordblks: i32,
    pub smblks: i32,
    pub hblks: i32,
    pub hblkhd: i32,
    pub usmblks: i32,
    pub fsmblks: i32,
    pub uordblks: i32,
    pub fordblks: i32,
    pub keepcost: i32,
}

/// C struct mallinfo2 layout (glibc >= 2.33)
/// unsigned 64-bit (size_t)
#[repr(C)]
#[derive(Debug, Clone, Copy)]
pub struct Mallinfo2C {
    pub arena: usize,
    pub ordblks: usize,
    pub smblks: usize,
    pub hblks: usize,
    pub hblkhd: usize,
    pub usmblks: usize,
    pub fsmblks: usize,
    pub uordblks: usize,
    pub fordblks: usize,
    pub keepcost: usize,
}

extern "C" {
    fn mallinfo2() -> Mallinfo2C;
}

fn clamp_to_int_max(value: i64) -> i32 {
    if value > i32::MAX as i64 {
        i32::MAX
    } else if value < 0 {
        0
    } else {
        value as i32
    }
}

/// Override mallinfo()
#[no_mangle]
pub extern "C" fn mallinfo() -> MallinfoC {
    static LOGGED_ONCE_ARENA_HBLKHD: AtomicBool = AtomicBool::new(false);
    static LOGGED_ONCE_UORDBLKS: AtomicBool = AtomicBool::new(false);
    let info2 = unsafe { mallinfo2() };

    // Clamp arena + hblkhd
    let mut arena = clamp_to_int_max(info2.arena as i64);
    let mut hblkhd = clamp_to_int_max(info2.hblkhd as i64);
    let sum = arena as i64 + hblkhd as i64;
    if sum > i32::MAX as i64 {
        // Scale both down proportionally
        // This maintains the ratio while ensuring sum <= INT_MAX
        let scale_factor = (i32::MAX as f64) / (sum as f64);
        arena = (arena as f64 * scale_factor) as i32;
        hblkhd = (hblkhd as f64 * scale_factor) as i32;

        if !LOGGED_ONCE_ARENA_HBLKHD.swap(true, Ordering::Relaxed) {
            warn!(
                arena = arena,
                hblkhd = hblkhd,
                sum = arena as i64 + hblkhd as i64,
                "arena + hblkhd > INT_MAX after clamping, scaling proportionally to prevent crash"
            );
        }
    }

    // Clamp uordblks
    let uordblks_i64 = info2.uordblks as i64;
    if uordblks_i64 > i32::MAX as i64 {
        if !LOGGED_ONCE_UORDBLKS.swap(true, Ordering::Relaxed) {
            warn!(
                uordblks = uordblks_i64,
                "uordblks > INT_MAX, clamping to prevent crash"
            );
        }
    }
    let uordblks = clamp_to_int_max(uordblks_i64);

    // Still clamp the other values for safety, though they aren't used by CEF
    MallinfoC {
        arena,
        ordblks: clamp_to_int_max(info2.ordblks as i64),
        smblks: clamp_to_int_max(info2.smblks as i64),
        hblks: clamp_to_int_max(info2.hblks as i64),
        hblkhd,
        usmblks: clamp_to_int_max(info2.usmblks as i64),
        fsmblks: clamp_to_int_max(info2.fsmblks as i64),
        uordblks,
        fordblks: clamp_to_int_max(info2.fordblks as i64),
        keepcost: clamp_to_int_max(info2.keepcost as i64),
    }
}

/// Constructor called when library is loaded
#[cfg(target_os = "linux")]
#[link_section = ".init_array"]
#[used]
static INIT: extern "C" fn() = init;

#[cfg(target_os = "linux")]
extern "C" fn init() {
    let _ = tracing_subscriber::fmt()
        .with_target(true)
        .with_thread_ids(true)
        .try_init();

    debug!("mallinfo-override loaded: using mallinfo2() with overflow protection");
}
