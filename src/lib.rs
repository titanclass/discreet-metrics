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
    fn write_desc(&mut self, desc: &MetricDesc);
    fn write(&mut self, bytes: &[u8]);
}

/// From OpenMetrics:
///
/// Metrics are a specific kind of telemetry data.
/// They represent a snapshot of the current state for a set of data.
/// They are distinct from logs or events, which focus on records or
/// information about individual events.
pub trait Metric {
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

    pub fn register(&self, item: NonNull<MetricDesc<'a>>) {
        let prev = self.head.load(Ordering::Relaxed);
        unsafe { item.as_ref().next.store(prev, Ordering::Relaxed) };
        self.head.store(item.as_ptr(), Ordering::Relaxed);
    }
}

impl<'a> Registry<'a> {
    // Collect the registered metrics and encode them
    pub fn encode(&self, enc: &mut dyn Encoder) {
        let mut n = &self.head;
        while let Some(i) = NonNull::new(n.load(Ordering::Relaxed)) {
            let item = unsafe { i.as_ref() };
            enc.write_desc(item);
            item.metric.encode(enc);
            n = &item.next;
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

        // The above line and the following can be done as a macro
        static mut METRIC_ITEM: MetricDesc =
            MetricDesc::new("some-metric", "Some metric", None, &["some-label"], &METRIC);
        REGISTRY.register(unsafe { NonNull::new(&mut METRIC_ITEM as *mut _).unwrap() });

        // This'll be what most people will have in the same file as the metric static
        METRIC.inc();

        // From elsewhere, we'd be establishing the encoder and outputting
        // its bytes somewhere either periodically or on demand.
        let mut encoder = MyEncoder;
        let _encoder = REGISTRY.encode(&mut encoder);
    }
}
