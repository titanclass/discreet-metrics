discreet-metrics
===

A metrics library aiming to conform with [OpenMetrics](https://openmetrics.io/) and satisfy the following goals:

* memory requirements for metrics are deterministic and fixed at compile time
* able to run on no-std targets as well as std
* sympathetic to hardware with limited support for atomics and floating types
* avoid the use of an allocator to support bare metal targets
