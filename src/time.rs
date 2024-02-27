use std::time::{Duration, Instant, SystemTime};

use crate::{check_concern, hostcalls, log_concern};

/// Fetches the realtime clock and stores it in a [`SystemTime`]
pub fn now() -> SystemTime {
    check_concern("now", hostcalls::get_current_time()).expect("failed to fetch realtime clock")
}

#[allow(dead_code)]
struct Timespec {
    tv_sec: i64,
    tv_nsec: u32,
}

/// Fetches the monotonic clock and stores it in an [`Instant`].
#[cfg(target_arch = "wasm32")]
pub fn instant_now() -> Instant {
    // proxy-wasm ignores precision
    let raw_ns: u64 = unsafe { wasi::clock_time_get(wasi::CLOCKID_MONOTONIC, 0) }
        .expect("failed to fetch monotonic time");
    debug_assert_eq!(
        std::mem::size_of::<Instant>(),
        std::mem::size_of::<Timespec>()
    );
    unsafe {
        std::mem::transmute::<Timespec, Instant>(Timespec {
            tv_sec: (raw_ns / 1000000000) as i64,
            tv_nsec: (raw_ns % 1000000000) as u32,
        })
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn instant_now() -> Instant {
    Instant::now()
}

/// Set tick period. Use `Duration::ZERO` to disable ticker.
pub fn set_tick_period(period: Duration) {
    log_concern("set-tick-period", hostcalls::set_tick_period(period));
}
