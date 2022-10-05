use std::sync::atomic::{AtomicBool, Ordering};

pub use tracer_macros::trace;

pub fn set_enabled(enabled: bool) {
    ENABLED.swap(enabled, Ordering::Relaxed);
}

pub fn enabled() -> bool {
    ENABLED.load(Ordering::Relaxed)
}

static ENABLED: AtomicBool = AtomicBool::new(false);
