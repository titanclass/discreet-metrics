//! The Prometheus text encoder adopted by OpenMetrics

use crate::{metrics::counter::Counter, Encoder, Metric};

pub struct TextEncoder;

impl Encoder for TextEncoder {
    fn write_desc(&mut self, _desc: &crate::MetricDesc)
    where
        Self: Sized,
    {
        // TODO
    }
    fn write(&mut self, _bytes: &[u8]) {
        // TODO
    }
}

impl Metric for Counter {
    fn encode(&self, _enc: &mut dyn Encoder) {
        // TODO
    }
}
