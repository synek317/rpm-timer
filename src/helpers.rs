use std::time::Instant;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub trait InstantExtensions {
    fn elapsed_seconds(&self) -> f64;
}

impl InstantExtensions for Instant {
    fn elapsed_seconds(&self) -> f64 {
        let elapsed = self.elapsed();

        elapsed.as_secs() as f64 + elapsed.subsec_nanos() as f64 / 1000_000_000f64
    }
}

pub trait AtomicUsizeExtensions {
    fn increase(&self);
    fn decrease(&self);
    fn get(&self) -> usize;
}

impl AtomicUsizeExtensions for Arc<AtomicUsize> {
    fn increase(&self) {
        self.fetch_add(1, Ordering::SeqCst);
    }

    fn decrease(&self) {
        self.fetch_sub(1, Ordering::SeqCst);
    }

    fn get(&self) -> usize {
        self.load(Ordering::SeqCst)
    }
}
