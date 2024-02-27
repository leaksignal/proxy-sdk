use std::{cell::RefCell, collections::HashMap};

use crate::{
    dispatcher::root_id,
    hostcalls::{self, MetricType},
    log_concern, Status,
};

#[derive(Default)]
pub struct MetricsInfo {
    counters: HashMap<String, u32>,
    gauges: HashMap<String, u32>,
    histograms: HashMap<String, u32>,
}

thread_local! {
    static METRICS: RefCell<HashMap<u32, MetricsInfo>> = RefCell::default();
}

/// Envoy counter metric handle
#[derive(Clone, Copy, Debug)]
pub struct Counter(u32);

/// Const wrapper for [`Counter`]
pub struct ConstCounter {
    name: &'static str,
}

impl ConstCounter {
    /// Const wrapper for [`Counter::define`]
    pub const fn define(name: &'static str) -> Self {
        Self { name }
    }

    pub fn get(&self) -> Counter {
        Counter::define(self.name)
    }
}

impl Counter {
    /// Defines a new counter, reusing an old handle if it already exists. It is safe to call this multiple times with the same name.
    pub fn define(name: impl AsRef<str>) -> Self {
        METRICS.with_borrow_mut(|metrics| {
            let metrics = metrics.entry(root_id()).or_default();
            if let Some(counter) = metrics.counters.get(name.as_ref()) {
                return Self(*counter);
            }
            let out = log_concern(
                "define-metric",
                hostcalls::define_metric(MetricType::Counter, name.as_ref()),
            );
            metrics.counters.insert(name.as_ref().to_string(), out);
            Self(out)
        })
    }

    /// Retrieves the current metric value
    pub fn get(&self) -> Result<u64, Status> {
        hostcalls::get_metric(self.0)
    }

    /// Records an absolute count of this metric
    pub fn record(&self, value: u64) {
        log_concern("record-metric", hostcalls::record_metric(self.0, value));
    }

    /// Increments the count of this metric by `offset`
    pub fn increment(&self, offset: i64) {
        log_concern(
            "increment-metric",
            hostcalls::increment_metric(self.0, offset),
        );
    }
}

/// Envoy gauge metric handle
#[derive(Clone, Copy, Debug)]
pub struct Gauge(u32);

/// Const wrapper for [`Gauge`]
pub struct ConstGauge {
    name: &'static str,
}

impl ConstGauge {
    /// Const wrapper for [`Gauge::define`]
    pub const fn define(name: &'static str) -> Self {
        Self { name }
    }

    pub fn get(&self) -> Gauge {
        Gauge::define(self.name)
    }
}

impl Gauge {
    /// Defines a new gauge, reusing an old handle if it already exists. It is safe to call this multiple times with the same name.
    pub fn define(name: impl AsRef<str>) -> Self {
        METRICS.with_borrow_mut(|metrics| {
            let metrics = metrics.entry(root_id()).or_default();
            if let Some(gauge) = metrics.gauges.get(name.as_ref()) {
                return Self(*gauge);
            }
            let out = log_concern(
                "define-metric",
                hostcalls::define_metric(MetricType::Gauge, name.as_ref()),
            );
            metrics.gauges.insert(name.as_ref().to_string(), out);
            Self(out)
        })
    }

    /// Retrieves the current metric value
    pub fn get(&self) -> Result<u64, Status> {
        hostcalls::get_metric(self.0)
    }

    /// Records an absolute count of this metric
    pub fn record(&self, value: u64) {
        log_concern("record-metric", hostcalls::record_metric(self.0, value));
    }

    /// Increments the count of this metric by `offset`
    pub fn increment(&self, offset: i64) {
        log_concern(
            "increment-metric",
            hostcalls::increment_metric(self.0, offset),
        );
    }
}

/// Envoy histogram metric handle
#[derive(Clone, Copy, Debug)]
pub struct Histogram(u32);

/// Const wrapper for [`Histogram`]
pub struct ConstHistogram {
    name: &'static str,
}

impl ConstHistogram {
    /// Const wrapper for [`Histogram::define`]
    pub const fn define(name: &'static str) -> Self {
        Self { name }
    }

    pub fn get(&self) -> Histogram {
        Histogram::define(self.name)
    }
}

impl Histogram {
    /// Defines a new histogram, reusing an old handle if it already exists. It is safe to call this multiple times with the same name.
    pub fn define(name: impl AsRef<str>) -> Self {
        METRICS.with_borrow_mut(|metrics| {
            let metrics = metrics.entry(root_id()).or_default();
            if let Some(histogram) = metrics.histograms.get(name.as_ref()) {
                return Self(*histogram);
            }
            let out = log_concern(
                "define-metric",
                hostcalls::define_metric(MetricType::Histogram, name.as_ref()),
            );
            metrics.histograms.insert(name.as_ref().to_string(), out);
            Self(out)
        })
    }

    /// Records a new item for this histogram
    pub fn record(&self, value: u64) {
        log_concern("record-metric", hostcalls::record_metric(self.0, value));
    }
}
