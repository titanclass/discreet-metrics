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
    /// Writes out the descriptor of a metric.
    fn write_desc(&mut self, desc: &MetricDesc);
    /// Called by a metric to encode itself.
    fn write(&mut self, bytes: &[u8]);
}

/// From OpenMetrics:
///
/// Metrics are a specific kind of telemetry data.
/// They represent a snapshot of the current state for a set of data.
/// They are distinct from logs or events, which focus on records or
/// information about individual events.
pub trait Metric {
    /// Encode this metric into a form expected by a given Encoder.
    fn encode(&self, enc: &mut dyn Encoder);
}

/// Enumerates the types of metrics as per OpenMetrics and what we
/// support
pub enum MetricType {
    Counter,
}

/// A metric descriptor exists for the purposes of registering a metric,
/// along with its meta data.
pub struct MetricDesc<'a> {
    pub name: &'a str,
    pub help: &'a str,
    pub unit: Option<&'a str>,
    pub labels: &'a [&'a str],

    metric: &'a dyn Metric,
    next: AtomicPtr<MetricDesc<'a>>,
}

impl<'a> MetricDesc<'a> {
    pub const fn new(
        name: &'a str,
        help: &'a str,
        unit: Option<&'a str>,
        labels: &'a [&'a str],
        metric: &'a dyn Metric,
    ) -> Self {
        Self {
            name,
            help,
            unit,
            labels,
            metric,
            next: AtomicPtr::new(ptr::null_mut()),
        }
    }
}

/// A registry retains a collection of metrics.
/// Metrics are retained in a chain of references
/// that must live at least as long as the registry
/// itself.
#[derive(Default)]
pub struct Registry<'a> {
    head: AtomicPtr<MetricDesc<'a>>,
}

impl<'a> Registry<'a> {
    pub const fn new() -> Self {
        Self {
            head: AtomicPtr::new(ptr::null_mut()),
        }
    }

    /// Register a metric descriptor. Registration is synchronized
    /// and so may therefore be called from multiple threads.
    pub fn register(&self, nonnull_desc_ptr: NonNull<MetricDesc<'a>>) {
        let desc = unsafe { nonnull_desc_ptr.as_ref() };
        let desc_ptr = nonnull_desc_ptr.as_ptr();

        loop {
            let head_desc_ptr = self.head.load(Ordering::Relaxed);
            let prev_desc_ptr = desc.next.swap(head_desc_ptr, Ordering::Relaxed);
            assert!(
                head_desc_ptr != desc_ptr && prev_desc_ptr.is_null(),
                "Metric is loaded more than once"
            );
            if self
                .head
                .compare_exchange(
                    head_desc_ptr,
                    desc_ptr,
                    Ordering::Relaxed,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                break;
            }
        }
    }
}

impl<'a> Registry<'a> {
    /// Collect the registered metrics and encode them
    pub fn encode(&self, enc: &mut dyn Encoder) {
        let mut next = &self.head;
        while let Some(nonnull_desc_ptr) = NonNull::new(next.load(Ordering::Relaxed)) {
            let desc = unsafe { nonnull_desc_ptr.as_ref() };
            enc.write_desc(desc);
            desc.metric.encode(enc);
            next = &desc.next;
        }
    }
}

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
        impl Metric for MyMetric {
            fn encode(&self, enc: &mut dyn Encoder) {
                enc.write(&self.count.load(Ordering::Relaxed).to_string().as_bytes());
            }
        }

        struct MyEncoder;
        impl Encoder for MyEncoder {
            fn write_desc(&mut self, desc: &MetricDesc)
            where
                Self: Sized,
            {
                assert_eq!(desc.name, "some-metric");
                assert_eq!(desc.help, "Some metric");
                assert!(desc.unit.is_none());
                assert_eq!(desc.labels, ["some-label"]);
            }

            fn write(&mut self, bytes: &[u8]) {
                assert_eq!(bytes, b"1");
            }
        }

        // A registry will be typically declared in a static
        static REGISTRY: Registry = Registry::new();

        // The user will declare a metric in their file, again as a static
        static METRIC: MyMetric = MyMetric::new();

        // The above line and the following can probably be done as a macro
        static mut METRIC_ITEM: MetricDesc =
            MetricDesc::new("some-metric", "Some metric", None, &["some-label"], &METRIC);

        // A metric desc can only be registered once and will panic otherwise!
        REGISTRY.register(unsafe { NonNull::new(&mut METRIC_ITEM as *mut _).unwrap() });

        // This'll be what most people will have in the same file as the metric static
        METRIC.inc();

        // From elsewhere, we'd be establishing the encoder and outputting
        // its bytes somewhere either periodically or on demand.
        let mut encoder = MyEncoder;
        let _encoder = REGISTRY.encode(&mut encoder);
    }
}
