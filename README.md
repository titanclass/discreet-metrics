\* \* \* EXPERIMENTAL \* \* \*

discreet-metrics
===

A high-performance/low-overhead metrics library aiming to conform with [OpenMetrics](https://openmetrics.io/) and to satisfy the following goals:

* memory requirements for metrics are deterministic and fixed at compile time
* able to run on no-std targets as well as std
* sympathetic to hardware with limited support for atomics and floating types
* avoids the use of an allocator in support of bare metal targets

As this library is intended to run on resource-constrained targets, you can expect good performance on all targets.

An example
---

```rust
use core::ptr::{ addr_of_mut, NonNull };
use discreet_metrics::{ encoders::text::TextEncoder, metrics::counter::Counter, MetricDesc, Registry };
use std::sync::Once;

// A registry will be typically declared in a static
static REGISTRY: Registry = Registry::new();

// Then referenced from within a file where it is required
// ```
// extern "Rust" {
//    static REGISTRY: Registry<'static>;
//}

// Declare a metric in a file where it is used, again as a static
static SOME_METRIC: Counter = Counter::new();
static mut SOME_METRIC_DESC: MetricDesc = 
    MetricDesc::new("some-metric", "Some metric", None, &SOME_METRIC);

// Register the metric descriptor - once, and only once! The following is achieved
// using std::sync::Once, but other methods including the lazy_static library can be
// used. This initialization would also typically appear within the file where the
// metric is used.
//
// The metric descriptor must also be declared to outlive the registry. This cannot be
// enforced.
static REGISTER_METRICS: Once = Once::new();
    
// later...
REGISTER_METRICS.call_once(|| {
    unsafe { REGISTRY.register(addr_of_mut!(SOME_METRIC_DESC)) };
});

// Do what we do with metric counters!
SOME_METRIC.inc();

// Elsewhere, establish the encoder and output its bytes somewhere 
// either periodically or on demand.
let mut encoder = TextEncoder;
let _encoder = REGISTRY.encode(&mut encoder);
```


## Contribution policy

Contributions via GitHub pull requests are gladly accepted from their original author. Along with any pull requests, please state that the contribution is your original work and that you license the work to the project under the project's open source license. Whether or not you state this explicitly, by submitting any copyrighted material via pull request, email, or other means you agree to license the material under the project's open source license and warrant that you have the legal authority to do so.

## License

This code is open source software licensed under the [Apache-2.0 license](./LICENSE).

© Copyright [Titan Class P/L](https://www.titanclass.com.au/), 2022
