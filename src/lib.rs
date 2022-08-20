#![doc = include_str!("../README.md")]
#![cfg_attr(not(test), no_std)]

use core::{
    ptr::{self, NonNull},
    sync::atomic::{AtomicPtr, Ordering},
};

pub mod encoders;
pub mod metrics;

/// An encoder encodes metrics into bytes.
pub trait Encoder {
    fn write(&mut self, bytes: &[u8]);
}

/// From OpenMetrics:
///
/// Metrics are a specific kind of telemetry data.
/// They represent a snapshot of the current state for a set of data.
/// They are distinct from logs or events, which focus on records or
/// information about individual events.
pub trait Metric<E: Encoder> {
    fn encode(&self, enc: E) -> E;
}

pub struct MetricItem<'a, E: Encoder> {
    metric: &'a dyn Metric<E>,
    next: AtomicPtr<MetricItem<'a, E>>,
}

impl<'a, E: Encoder> MetricItem<'a, E> {
    pub const fn new(metric: &'a dyn Metric<E>) -> Self {
        Self {
            metric,
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

/// A registry retains a collection of metrics.
/// Metrics are retained in a chain of references
/// that must live at least as long as the registry
/// itself.
pub struct Registry<'a, E: Encoder> {
    head: AtomicPtr<MetricItem<'a, E>>,
}

impl<'a, E: Encoder> Registry<'a, E> {
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    pub fn register(&self, item: NonNull<MetricItem<'a, E>>) {
        let prev = self.head.load(Ordering::Relaxed);
        unsafe { item.as_ref().next.store(prev, Ordering::Relaxed) };
        self.head.store(item.as_ptr(), Ordering::Relaxed);
    }
}

impl<'a, E: Encoder> Registry<'a, E> {
    // Collect the registered metrics and encode them
    pub fn encode(&self, mut enc: E) -> E {
        let mut n = &self.head;
        while let Some(i) = NonNull::new(n.load(Ordering::Relaxed)) {
            let item = unsafe { i.as_ref() };
            enc = item.metric.encode(enc);
            n = &item.next;
        }
        enc
    }
}

impl<'a, E: Encoder> Default for Registry<'a, E> {
    fn default() -> Self {
        Self {
            head: Default::default(),
        }
    }
}

impl<'a, E: Encoder> Registry<'a, E> {}

#[cfg(test)]
mod tests {

    use core::sync::atomic::AtomicUsize;

    use super::*;

    #[test]
    fn registration() {
        // This will be provided by the library

        #[derive(Default)]
        struct MyMetric {
            count: AtomicUsize,
        }
        impl MyMetric {
            const fn new() -> Self {
                Self {
                    count: AtomicUsize::new(0),
                }
            }

            fn inc(&self) {
                self.count.fetch_add(1, Ordering::Relaxed);
            }
        }
        impl<E: Encoder> Metric<E> for MyMetric {
            fn encode(&self, mut enc: E) -> E {
                enc.write(&self.count.load(Ordering::Relaxed).to_string().as_bytes());
                enc
            }
        }

        struct MyEncoder;
        impl Encoder for MyEncoder {
            fn write(&mut self, bytes: &[u8]) {
                assert_eq!(bytes, b"1");
            }
        }

        // A registry will typically declared in a static
        static REGISTRY: Registry<MyEncoder> = Registry::new();

        // The user will declare a metric in their file, again as a static
        static METRIC: MyMetric = MyMetric::new();

        // The above line and the following can be done as a macro
        let mut metric_item = MetricItem::new(&METRIC);
        REGISTRY.register(NonNull::new(&mut metric_item as *mut _).unwrap());

        // This'll be what most people will have in the same file as the metric static
        METRIC.inc();

        // From elsewhere, we'd be establishing the encoder and outputting
        // its bytes somewhere either periodically or on demand.
        let encoder = MyEncoder;
        let _encoder = REGISTRY.encode(encoder);
    }
}
