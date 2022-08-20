//! The Prometheus text encoder adopted by OpenMetrics

use crate::{metrics::counter::Counter, Encoder, Metric};

pub struct TextEncoder;

impl Encoder for TextEncoder {
    fn write_desc(&mut self, _desc: &crate::MetricDesc<Self>)
    where
        Self: Sized,
    {
        // TODO
    }
    fn write(&mut self, _bytes: &[u8]) {
        // TODO
    }
}

impl Metric<TextEncoder> for Counter {
    fn encode(&self, enc: TextEncoder) -> TextEncoder {
        // TODO
        enc
    }
}
