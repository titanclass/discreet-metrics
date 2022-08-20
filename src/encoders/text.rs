//! The Prometheus text encoder adopted by OpenTelemetry

use crate::{metrics::counter::Counter, Encoder, Metric};

struct TextEncoder;

impl Encoder for TextEncoder {
    fn write(&mut self, _bytes: &[u8]) {
        todo!()
    }
}

impl Metric<TextEncoder> for Counter {
    fn encode(&self, _enc: TextEncoder) -> TextEncoder {
        todo!()
    }
}
