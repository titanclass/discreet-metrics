discreet-metrics
===

A high-performance/low-overhead metrics library aiming to conform with [OpenMetrics](https://openmetrics.io/) and satisfy the following goals:

* memory requirements for metrics are deterministic and fixed at compile time
* able to run on no-std targets as well as std
* sympathetic to hardware with limited support for atomics and floating types
* avoid the use of an allocator to support bare metal targets

As this library is intended to run on resource-constrained targets, you can expect good performance on targets in general.

An example
---

```rust
use core::ptr::NonNull;
use discreet_metrics::{ encoders::text::TextEncoder, metrics::counter::Counter, MetricDesc, Registry };

// A registry will be typically declared in a static
static REGISTRY: Registry<TextEncoder> = Registry::new();

// Declare a metric in a file where it is used, again as a static
static METRIC: Counter = Counter::new();

// Register the metric
static mut METRIC_ITEM: MetricDesc<TextEncoder> =
    MetricDesc::new("some-metric", "Some metric", None, &METRIC);
REGISTRY.register(unsafe { NonNull::new(&mut METRIC_ITEM as *mut _).unwrap() });

// Do what we do with metric counters!
METRIC.inc();

// Elsewhere, establish the encoder and output its bytes somewhere 
// either periodically or on demand.
let encoder = TextEncoder;
let _encoder = REGISTRY.encode(encoder);
```


## Contribution policy

Contributions via GitHub pull requests are gladly accepted from their original author. Along with any pull requests, please state that the contribution is your original work and that you license the work to the project under the project's open source license. Whether or not you state this explicitly, by submitting any copyrighted material via pull request, email, or other means you agree to license the material under the project's open source license and warrant that you have the legal authority to do so.

## License

This code is open source software licensed under the [Apache-2.0 license](./LICENSE).

© Copyright [Titan Class P/L](https://www.titanclass.com.au/), 2022
