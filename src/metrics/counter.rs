//! From OpenTelemetry:
//!
//! Counters measure discrete events. Common examples are the number of HTTP requests received,
//! CPU seconds spent, or bytes sent. For counters how quickly they are increasing over time
//! is what is of interest to a user.

use core::sync::atomic::{AtomicUsize, Ordering};

#[derive(Default)]
pub struct Counter {
    total: AtomicUsize,
}

impl Counter {
    pub const fn new() -> Self {
        Self {
            total: AtomicUsize::new(0),
        }
    }

    /// Add one to the counter
    pub fn inc(&self) {
        self.total.fetch_add(1, Ordering::Relaxed);
    }

    /// Add a number of counts to the counter
    pub fn inc_by(&self, count: usize) {
        self.total.fetch_add(count, Ordering::Relaxed);
    }

    /// Return the current total
    pub fn total(&self) -> usize {
        self.total.load(Ordering::Relaxed)
    }
}
